use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, Error, ReadBuf};
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;

use std::io;

use crate::Addr;

/// A socket connected to an endpoint
#[derive(Debug)]
pub enum Stream {
    /// A TCP stream between a local and a remote socket.
    Inet(TcpStream),
    #[cfg(unix)]
    /// A connected Unix socket
    Unix(UnixStream),
}

impl From<TcpStream> for Stream {
    fn from(s: TcpStream) -> Stream {
        Stream::Inet(s)
    }
}

#[cfg(unix)]
impl From<UnixStream> for Stream {
    fn from(s: UnixStream) -> Stream {
        Stream::Unix(s)
    }
}

impl Stream {
    /// Opens a connection to a remote host.
    pub async fn connect(s: &Addr) -> io::Result<Stream> {
        match s {
            Addr::Inet(s) => TcpStream::connect(s).await.map(Stream::Inet),
            #[cfg(unix)]
            Addr::Unix(s) => UnixStream::connect(s).await.map(Stream::Unix),
        }
    }

    /// Returns the local address that this stream is bound to.
    pub fn local_addr(&self) -> io::Result<Addr> {
        match self {
            Stream::Inet(s) => s.local_addr().map(Addr::Inet),
            #[cfg(unix)]
            Stream::Unix(s) => s.local_addr().map(|e| e.into()),
        }
    }

    /// Returns the remote address that this stream is connected to.
    pub fn peer_addr(&self) -> io::Result<Addr> {
        match self {
            Stream::Inet(s) => s.peer_addr().map(Addr::Inet),
            #[cfg(unix)]
            Stream::Unix(s) => s.peer_addr().map(|e| e.into()),
        }
    }
}
impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Error>> {
        match &mut *self {
            Stream::Inet(s) => Pin::new(s).as_mut().poll_read(cx, buf),
            #[cfg(unix)]
            Stream::Unix(s) => Pin::new(s).as_mut().poll_read(cx, buf),
        }
    }
}
impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        match &mut *self {
            Stream::Inet(s) => Pin::new(s).as_mut().poll_write(cx, buf),
            #[cfg(unix)]
            Stream::Unix(s) => Pin::new(s).as_mut().poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        match &mut *self {
            Stream::Inet(s) => Pin::new(s).as_mut().poll_flush(cx),
            #[cfg(unix)]
            Stream::Unix(s) => Pin::new(s).as_mut().poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        match &mut *self {
            Stream::Inet(s) => Pin::new(s).as_mut().poll_shutdown(cx),
            #[cfg(unix)]
            Stream::Unix(s) => Pin::new(s).as_mut().poll_shutdown(cx),
        }
    }
}



#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    #[cfg(unix)]
    use tokio::net::UnixListener;
    use tokio::runtime::Builder;

    pub(crate) async fn local_socket_pair() -> Result<(TcpListener, Addr), std::io::Error> {
        let a: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let app_listener = TcpListener::bind(a).await?;
        let a: Addr = app_listener.local_addr()?.into();
        Ok((app_listener, a))
    }

    #[test]
    fn tcp_connect() {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        async fn mock_app(app_listener: TcpListener) {
            let (mut app_socket, _) = app_listener.accept().await.unwrap();
            let mut buf = [0u8; 32];
            let i = app_socket.read(&mut buf).await.unwrap();
            app_socket.write_all(&buf[..i]).await.unwrap();
        }

        async fn con() {
            let (app_listener, a) = local_socket_pair().await.unwrap();
            tokio::spawn(mock_app(app_listener));

            let mut s = Stream::connect(&a).await.expect("tcp connect failed");

            let data = b"1234";
            s.write_all(&data[..]).await.expect("tcp write failed");

            let mut buf = [0u8; 32];
            let i = s.read(&mut buf).await.expect("tcp read failed");
            assert_eq!(&buf[..i], &data[..]);
        }
        rt.block_on(con());
    }
    #[cfg(unix)]
    #[test]
    fn unix_connect() {
        use std::path::Path;

        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        async fn mock_app(app_listener: UnixListener) {
            let (mut app_socket, _) = app_listener.accept().await.unwrap();
            let mut buf = [0u8; 32];
            let i = app_socket.read(&mut buf).await.unwrap();
            app_socket.write_all(&buf[..i]).await.unwrap();
        }

        async fn con() {
            let a: &Path = Path::new("/tmp/afcgi.sock");
            let app_listener = UnixListener::bind(a).unwrap();
            tokio::spawn(mock_app(app_listener));

            let a: Addr = "/tmp/afcgi.sock".parse().expect("unix parse failed");
            let mut s = Stream::connect(&a).await.expect("unix connect failed");

            let data = b"1234";
            s.write_all(&data[..]).await.expect("unix write failed");

            let mut buf = [0u8; 32];
            let i = s.read(&mut buf).await.expect("unix read failed");
            assert_eq!(&buf[..i], &data[..]);
        }
        rt.block_on(con());
        std::fs::remove_file("/tmp/afcgi.sock").unwrap();
    }
}
