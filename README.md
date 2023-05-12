[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![crates.io](https://img.shields.io/crates/v/async-stream-connection.svg)](https://crates.io/crates/async-stream-connection)
[![Released API docs](https://docs.rs/async-stream-connection/badge.svg)](https://docs.rs/async-stream-connection)
[![GitHub](https://img.shields.io/github/license/User65k/async-stream-connection)](./LICENSE)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/User65k/async-stream-connection/ubuntu.yml)

A simple enum that supports `tokio::io::AsyncRead` and `tokio::io::AsyncWrite` on TCP as well as Unix sockets.