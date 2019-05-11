use futures::io::{AsyncRead, AsyncWrite};
use futures::task::{Context, Poll};
use pin_utils::unsafe_pinned;
use romio::tcp::TcpStream;

use std::io;
use std::pin::Pin;

pub struct NetStream {
    stream: TcpStream,
}

impl NetStream {
    unsafe_pinned!(stream: TcpStream);

    pub(crate) fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl AsyncRead for NetStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.stream().poll_read(cx, buf)
    }
}

impl AsyncWrite for NetStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.as_mut().stream().poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.as_mut().stream().poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.as_mut().stream().poll_close(cx)
    }
}
