# qusb2snes-client ![License: MIT](https://img.shields.io/badge/license-MIT-blue) [![qusb2snes-client on crates.io](https://img.shields.io/crates/v/qusb2snes-client)](https://crates.io/crates/qusb2snes-client) [![qusb2snes-client on docs.rs](https://docs.rs/qusb2snes-client/badge.svg)](https://docs.rs/qusb2snes-client) [![Source Code Repository](https://img.shields.io/badge/Code-On%20github.com-blue)](https://github.com/jhelwig/qusb2snes-client) [![qusb2snes-client on deps.rs](https://deps.rs/repo/github/jhelwig/qusb2snes-client/status.svg)](https://deps.rs/repo/github/jhelwig/qusb2snes-client)

This crate is currently **experimental**.

This crate allows interfacing with the websocket server provided by [Qusb2snes][__link0].


## Examples


```rust
let mut client = Client::new().await.unwrap();
println!("{:#?}", client.device_list().await);
```


 [__link0]: http://usb2snes.com/
