//! Signal disconnect before the MCP SDK drains in-flight handler responses.
use std::{
    io::{self, IoSlice},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::watch,
};

pub(super) struct DisconnectIo<T> {
    inner: T,
    shutdown: watch::Sender<bool>,
}

impl<T> DisconnectIo<T> {
    pub(super) fn new(inner: T, shutdown: watch::Sender<bool>) -> Self {
        Self { inner, shutdown }
    }

    fn disconnected(&self) {
        let _ = self.shutdown.send(true);
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for DisconnectIo<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buffer.filled().len();
        let has_capacity = buffer.remaining() > 0;
        let result = Pin::new(&mut self.inner).poll_read(cx, buffer);
        if matches!(&result, Poll::Ready(Err(_)))
            || (matches!(&result, Poll::Ready(Ok(())))
                && has_capacity
                && buffer.filled().len() == before)
        {
            self.disconnected();
        }
        result
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for DisconnectIo<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        let result = Pin::new(&mut self.inner).poll_write(cx, buffer);
        if matches!(&result, Poll::Ready(Err(_)))
            || (!buffer.is_empty() && matches!(&result, Poll::Ready(Ok(0))))
        {
            self.disconnected();
        }
        result
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffers: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let result = Pin::new(&mut self.inner).poll_write_vectored(cx, buffers);
        if matches!(&result, Poll::Ready(Err(_)))
            || (buffers.iter().any(|buffer| !buffer.is_empty())
                && matches!(&result, Poll::Ready(Ok(0))))
        {
            self.disconnected();
        }
        result
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.inner).poll_flush(cx);
        if matches!(&result, Poll::Ready(Err(_))) {
            self.disconnected();
        }
        result
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.inner).poll_shutdown(cx);
        if result.is_ready() {
            self.disconnected();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn nonempty_read_eof_signals_shutdown_without_waiting_for_sdk_drain() {
        let (mut host, stream) = tokio::io::duplex(64);
        let (stop, shutdown) = watch::channel(false);
        let mut stream = DisconnectIo::new(stream, stop);
        host.write_all(b"request").await.unwrap();
        assert_eq!(stream.read(&mut []).await.unwrap(), 0);
        assert!(!*shutdown.borrow(), "an empty read is not EOF");
        let mut request = [0; 7];
        stream.read_exact(&mut request).await.unwrap();
        assert_eq!(&request, b"request");
        assert!(!*shutdown.borrow());
        host.shutdown().await.unwrap();
        assert_eq!(stream.read(&mut request).await.unwrap(), 0);
        assert!(
            *shutdown.borrow(),
            "EOF must signal before poll_read returns"
        );
    }

    #[tokio::test]
    async fn broken_response_pipe_signals_shutdown() {
        let (host, stream) = tokio::io::duplex(64);
        let (stop, shutdown) = watch::channel(false);
        let mut stream = DisconnectIo::new(stream, stop);
        drop(host);
        assert!(stream.write_all(b"response").await.is_err());
        assert!(*shutdown.borrow());
    }

    struct FailedIo;
    impl AsyncRead for FailedIo {
        fn poll_read(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "fixture disconnect",
            )))
        }
    }
    impl AsyncWrite for FailedIo {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &[u8],
        ) -> Poll<io::Result<usize>> {
            Poll::Ready(Ok(0))
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "fixture disconnect",
            )))
        }
        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "fixture disconnect",
            )))
        }
    }

    #[tokio::test]
    async fn read_write_flush_and_shutdown_failures_signal_before_returning() {
        for operation in ["read", "write", "vectored", "flush", "shutdown"] {
            let (stop, shutdown) = watch::channel(false);
            let mut stream = DisconnectIo::new(FailedIo, stop);
            match operation {
                "read" => assert!(stream.read(&mut [0; 1]).await.is_err()),
                "write" => assert_eq!(stream.write(b"x").await.unwrap(), 0),
                "vectored" => assert_eq!(
                    stream.write_vectored(&[IoSlice::new(b"x")]).await.unwrap(),
                    0
                ),
                "flush" => assert!(stream.flush().await.is_err()),
                "shutdown" => assert!(stream.shutdown().await.is_err()),
                _ => unreachable!(),
            }
            assert!(*shutdown.borrow(), "{operation} failed to signal shutdown");
        }
    }
}
