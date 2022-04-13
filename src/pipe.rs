use std::{
    io,
    pin::Pin,
    task::{Context, Poll}, os::windows::prelude::{AsRawHandle, RawHandle},
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::windows::named_pipe::{NamedPipeClient, NamedPipeServer},
};

pub enum NamedPipe {
    Client(NamedPipeClient),
    Server(NamedPipeServer),
}

impl AsyncRead for NamedPipe {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match Pin::into_inner(self) {
            NamedPipe::Client(ref mut x) => Pin::new(x).poll_read(cx, buf),
            NamedPipe::Server(ref mut x) => Pin::new(x).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for NamedPipe {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match Pin::into_inner(self) {
            NamedPipe::Client(ref mut x) => Pin::new(x).poll_write(cx, buf),
            NamedPipe::Server(ref mut x) => Pin::new(x).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match Pin::into_inner(self) {
            NamedPipe::Client(ref mut x) => Pin::new(x).poll_flush(cx),
            NamedPipe::Server(ref mut x) => Pin::new(x).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match Pin::into_inner(self) {
            NamedPipe::Client(ref mut x) => Pin::new(x).poll_shutdown(cx),
            NamedPipe::Server(ref mut x) => Pin::new(x).poll_shutdown(cx),
        }
    }
}

impl AsRawHandle for NamedPipe {
    fn as_raw_handle(&self) -> RawHandle {
        match self {
            NamedPipe::Client(x) => x.as_raw_handle(),
            NamedPipe::Server(x) => x.as_raw_handle(),
        }
    }
}
