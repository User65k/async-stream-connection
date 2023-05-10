use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;

use std::io;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

use crate::{Addr, Stream};

/// A socket server, listening for connections.
///
/// You can accept a new connection by using the [`Listener::accept`] method.
pub enum Listener {
    /// A TCP socket server, listening for connections.
    Inet(TcpListener),
    #[cfg(unix)]
    /// A Unix socket which can accept connections from other Unix sockets.
    Unix(UnixListener),
}
impl Listener {
    /// Creates a new Listener, which will be bound to the specified address.
    ///
    /// The returned listener is ready for accepting connections.
    pub async fn bind(s: &Addr) -> io::Result<Listener> {
        match s {
            Addr::Inet(s) => TcpListener::bind(s).await.map(Listener::Inet),
            #[cfg(unix)]
            Addr::Unix(s) => UnixListener::bind(s).map(Listener::Unix),
        }
    }
    /// Accepts a new incoming connection from this listener.
    /// 
    /// This function will yield once a new connection is established.
    /// When established, the corresponding [`Stream`] and the remote peer’s address will be returned.
    pub async fn accept(&self) -> io::Result<(Stream, Addr)> {
        match self {
            Listener::Inet(s) => s
                .accept()
                .await
                .map(|(s, a)| (Stream::Inet(s), Addr::Inet(a))),
            #[cfg(unix)]
            Listener::Unix(s) => s
                .accept()
                .await
                .map(|(s, a)| (Stream::Unix(s), Addr::from(a))),
        }
    }
}
#[cfg(unix)]
impl AsRawFd for Listener {
    fn as_raw_fd(&self) -> RawFd {
        match self {
            Listener::Inet(s) => s.as_raw_fd(),
            #[cfg(unix)]
            Listener::Unix(s) => s.as_raw_fd(),
        }
    }
}
#[cfg(unix)]
impl Drop for Listener {
    fn drop(&mut self) {
        if let Listener::Unix(l) = self {
            if let Ok(a) = l.local_addr() {
                if let Some(path) = a.as_pathname() {
                    std::fs::remove_file(path).unwrap();
                }
            }
        }
    }
}