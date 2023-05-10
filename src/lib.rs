/*! A simple enum that supports [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`] on TCP as well as Unix sockets.
 *
 *
 *  # Example
 * 
 *  A simple TCP echo server:
 * 
 *  ```no_run
 *  use async_stream_connection::Listener;
 *  use tokio::io::{AsyncReadExt, AsyncWriteExt};
 * 
 *  #[tokio::main(flavor = "current_thread")]
 *  async fn main() -> Result<(), Box<dyn std::error::Error>> {
 *      let listener = Listener::bind(&"127.0.0.1:8080".parse()?).await?;
 * 
 *      loop {
 *          let (mut socket, _) = listener.accept().await?;
 * 
 *          tokio::spawn(async move {
 *              let mut buf = [0; 1024];
 * 
 *              // In a loop, read data from the socket and write the data back.
 *              loop {
 *                  let n = match socket.read(&mut buf).await {
 *                      // socket closed
 *                      Ok(n) if n == 0 => return,
 *                      Ok(n) => n,
 *                      Err(e) => {
 *                          eprintln!("failed to read from socket; err = {:?}", e);
 *                          return;
 *                      }
 *                  };
 * 
 *                  // Write the data back
 *                  if let Err(e) = socket.write_all(&buf[0..n]).await {
 *                      eprintln!("failed to write to socket; err = {:?}", e);
 *                      return;
 *                  }
 *              }
 *          });
 *      }
 *  }
 *  ```
 */
#![cfg_attr(docsrs, feature(doc_cfg))]

mod addr;
mod stream;
mod listener;

pub use addr::Addr;
pub use stream::Stream;
pub use listener::Listener;

