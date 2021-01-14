use std::mem::MaybeUninit;
use std::os::windows::io::AsRawHandle;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

#[repr(transparent)]
pub struct NamedPipeStream(pub(crate) tokio::io::PollEvented<mio_named_pipes::NamedPipe>);

impl AsRawHandle for NamedPipeStream {
    fn as_raw_handle(&self) -> std::os::windows::prelude::RawHandle {
        self.0.get_ref().as_raw_handle()
    }
}

impl AsyncRead for NamedPipeStream {
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
        self.0.prepare_uninitialized_buffer(buf)
    }

    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_read(ctx, buf)
    }
}

impl AsyncWrite for NamedPipeStream {
    fn poll_write(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_write(ctx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_flush(ctx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_shutdown(ctx)
    }
}

#[cfg(feature = "tonic")]
impl tonic::transport::server::Connected for NamedPipeStream {}
