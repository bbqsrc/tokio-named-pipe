#![cfg(windows)]

pub mod secattr;
mod stream;
pub use stream::NamedPipeStream;

use std::os::windows::io::{FromRawHandle, IntoRawHandle};
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio::future::poll_fn;

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

pub struct NamedPipeListener<'a> {
    path: PathBuf,
    config: NamedPipeConfig<'a>,
    io: tokio::io::PollEvented<mio_named_pipes::NamedPipe>,
}

pub struct NamedPipeConfig<'s> {
    pub inbound: bool,
    pub outbound: bool,
    pub out_buffer_size: u32,
    pub in_buffer_size: u32,
    pub security_attributes: secattr::SecurityAttributes<'s>,
}

impl<'s> Default for NamedPipeConfig<'s> {
    fn default() -> Self {
        NamedPipeConfig {
            inbound: true,
            outbound: true,
            out_buffer_size: 0x10000,
            in_buffer_size: 0x10000,
            security_attributes: Default::default(),
        }
    }
}

impl<'s> NamedPipeListener<'s> {
    fn new_raw(
        path: &std::path::Path,
        config: &mut NamedPipeConfig,
        is_first: bool,
    ) -> std::io::Result<mio_named_pipes::NamedPipe> {
        // mio-named-pipe doesn't allow configuration, as described in its documentation,
        // so we create a pipe here and put it into the rest of the lifecycle.
        let raw_handle = unsafe {
            miow::pipe::NamedPipeBuilder::new(&path)
                .first(is_first)
                .inbound(config.inbound)
                .outbound(config.outbound)
                .out_buffer_size(config.out_buffer_size)
                .in_buffer_size(config.in_buffer_size)
                .with_security_attributes(&mut config.security_attributes as *mut _ as *mut _)?
                .into_raw_handle()
        };

        let mio_pipe = unsafe { mio_named_pipes::NamedPipe::from_raw_handle(raw_handle) };
        Ok(mio_pipe)
    }

    pub fn bind<P: AsRef<std::path::Path>>(
        path: P,
        config: Option<NamedPipeConfig<'s>>,
    ) -> std::io::Result<NamedPipeListener> {
        let mut config = config.unwrap_or_default();
        let raw = Self::new_raw(&path.as_ref(), &mut config, true)?;
        Ok(NamedPipeListener {
            path: path.as_ref().to_path_buf(),
            config,
            io: tokio::io::PollEvented::new(raw)?,
        })
    }

    pub async fn accept<'a>(&'a mut self) -> tokio::io::Result<(stream::NamedPipeStream, PathBuf)> {
        poll_fn(|cx| self.poll_accept(cx)).await
    }

    pub(crate) fn poll_accept(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<tokio::io::Result<(stream::NamedPipeStream, PathBuf)>> {
        match self.io.get_ref().connect() {
            Ok(()) => {
                log::trace!("Incoming connection polled successfully");

                let raw = Self::new_raw(&self.path, &mut self.config, false)?;
                let raw = tokio::io::PollEvented::new(raw)?;

                let new_stream = NamedPipeStream(std::mem::replace(&mut self.io, raw));

                Poll::Ready(Ok((new_stream, self.path.to_path_buf())))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                self.io.clear_write_ready(cx)?;
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    pub fn incoming<'a>(&'a mut self) -> Incoming<'a, 's> {
        Incoming::new(self)
    }
}

/// Stream of incoming connections
pub struct Incoming<'a, 's> {
    inner: &'a mut NamedPipeListener<'s>,
}

impl<'a, 's> Incoming<'a, 's> {
    pub(crate) fn new(listener: &'a mut NamedPipeListener<'s>) -> Incoming<'a, 's> {
        Incoming { inner: listener }
    }

    /// Attempts to poll `NamedPipeStream` by polling inner `NamedPipeListener` to accept
    /// connection.
    ///
    /// If `NamedPipeListener` isn't ready yet, `Poll::Pending` is returned and
    /// current task will be notified by a waker.  Otherwise `Poll::Ready` with
    /// `Result` containing `NamedPipeStream` will be returned.
    pub fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<tokio::io::Result<NamedPipeStream>> {
        let (socket, _) = ready!(self.inner.poll_accept(cx))?;
        Poll::Ready(Ok(socket))
    }
}

impl<'a, 's> Stream for Incoming<'a, 's> {
    type Item = tokio::io::Result<NamedPipeStream>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (socket, _) = ready!(self.inner.poll_accept(cx))?;
        Poll::Ready(Some(Ok(socket)))
    }
}
