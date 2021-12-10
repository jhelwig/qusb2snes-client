# qusb2snes-client

This crate is currently **experimental**.

This crate allows interfacing with the websocket server provided by
[Qusb2snes][qusb2snes].

[qusb2snes]: http://usb2snes.com/

## Examples

```rust
let mut client = Client::new().await.unwrap();
println!("{:#?}", client.device_list().await);
```

License: MIT
