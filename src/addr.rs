use std::fmt;
use std::net;
use std::net::ToSocketAddrs;
#[cfg(unix)]
use std::os::unix::net as unix;
#[cfg(unix)]
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Address of a Stream Endpoint
/// ```
/// # use async_stream_connection::Addr;
/// # fn main() -> Result<(),std::io::Error> {
/// let addr: Addr = "127.0.0.1:1337".parse()?;
/// # Ok(())
/// # }
/// ```
/// or (unix only):
/// ```
/// # use async_stream_connection::Addr;
/// # fn main() -> Result<(),std::io::Error> {
/// # #[cfg(unix)]
/// let addr: Addr = "/tmp/uds_example".parse()?;
/// # Ok(())
/// # }
/// ```
/// [`FromStr::parse`] / Deserialize also resolves to the first IP Address if it does not start with `/` or `./`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Addr {
    /// An IP socket address
    Inet(net::SocketAddr),
    #[cfg(unix)]
    #[cfg_attr(docsrs, doc(cfg(unix)))]
    ///A UDS address
    Unix(PathBuf),
}

impl From<net::SocketAddr> for Addr {
    fn from(s: net::SocketAddr) -> Addr {
        Addr::Inet(s)
    }
}

#[cfg(unix)]
impl From<&Path> for Addr {
    fn from(s: &Path) -> Addr {
        Addr::Unix(s.to_path_buf())
    }
}
#[cfg(unix)]
impl From<PathBuf> for Addr {
    fn from(s: PathBuf) -> Addr {
        Addr::Unix(s)
    }
}
#[cfg(unix)]
impl From<unix::SocketAddr> for Addr {
    fn from(s: unix::SocketAddr) -> Addr {
        Addr::Unix(match s.as_pathname() {
            None => Path::new("unnamed").to_path_buf(),
            Some(p) => p.to_path_buf(),
        })
    }
}
#[cfg(unix)]
impl From<tokio::net::unix::SocketAddr> for Addr {
    fn from(s: tokio::net::unix::SocketAddr) -> Addr {
        Addr::Unix(match s.as_pathname() {
            None => Path::new("unnamed").to_path_buf(),
            Some(p) => p.to_path_buf(),
        })
    }
}
impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Addr::Inet(n) => n.fmt(f),
            #[cfg(unix)]
            Addr::Unix(n) => n.to_string_lossy().fmt(f),
        }
    }
}

impl FromStr for Addr {
    type Err = std::io::Error;

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        #[cfg(unix)]
        if v.starts_with('/') || v.starts_with("./") {
            return Ok(Addr::Unix(PathBuf::from(v)));
        }
        match v.to_socket_addrs()?.next() {
            Some(a) => Ok(Addr::Inet(a)),
            None => Err(std::io::ErrorKind::AddrNotAvailable.into())
        }        
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> serde::de::Deserialize<'de> for Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        //let s = Deserialize::<&str>::deserialize(deserializer)?;
        //Addr::from_str(&s).map_err(de::Error::custom)
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Addr;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Socket Address")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Addr::from_str(v).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn parse_addr() {
        assert!(if let Ok(Addr::Inet(net::SocketAddr::V4(f))) = Addr::from_str("127.0.0.1:9000") {
            f.ip().is_loopback() && f.port() == 9000
        }else{
            false
        });
        assert!(if let Ok(Addr::Inet(f)) = Addr::from_str("localhost:9000") {
            f.port() == 9000
        }else{
            println!("{:?}", Addr::from_str("localhost:9000"));
            false
        });
        assert!(if let Ok(Addr::Inet(net::SocketAddr::V6(f))) = Addr::from_str("[::1]:9000") {
            f.ip().is_loopback() && f.port() == 9000
        }else{
            false
        });
        #[cfg(unix)]
        assert!(if let Ok(Addr::Unix(f)) = Addr::from_str("/path") {
            f == std::path::Path::new("/path")
        }else{
            false
        });
    }
    #[test]
    fn display() {
        assert_eq!(
            "127.0.0.1:1234",
            Addr::Inet(net::SocketAddr::V4(net::SocketAddrV4::new(
                net::Ipv4Addr::new(127, 0, 0, 1),
                1234
            )))
            .to_string()
        );
        #[cfg(unix)]
        assert_eq!(
            "/tmp/bla",
            Addr::Unix(PathBuf::from_str("/tmp/bla").unwrap()).to_string()
        );
    }
}
