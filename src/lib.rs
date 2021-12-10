#![warn(clippy::all, clippy::pedantic)]
//! This crate is currently **experimental**.
//!
//! This crate allows interfacing with the websocket server provided by
//! [Qusb2snes][qusb2snes].
//! 
//! # Examples
//! 
//! ```ignore
//! # // TODO: Come up with a testable example.
//! let mut client = Client::new().await.unwrap();
//! println!("{:#?}", client.device_list().await);
//! ```
//!
//! [qusb2snes]: http://usb2snes.com/


pub mod results;
pub mod request;
pub mod offsets;

use websockets::{
    Frame,
    WebSocket,
};
pub use results::{
    Result,
    ResultData,
};
pub use request::Request;

pub struct Client {
    websocket: WebSocket,
}

impl Client {
    /// # Errors
    /// 
    /// Will return [`Err`] if the underlying [`websockets::WebSocket`] returns an [`Err`].
    pub async fn new() -> std::result::Result<Self, websockets::WebSocketError> {
        let websocket = WebSocket::connect("ws://localhost:8080").await?;

        Ok(Client { websocket })
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    pub async fn device_list(&mut self) -> Result {
        let request_string = serde_json::to_string(&Request::device_list()).unwrap();
        self.send_text(&request_string).await;

        match self.websocket.receive().await {
            Ok(res) => self.deserialize_response(res),
            Err(e) => panic!("{:#?}", e),
        }
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    pub async fn attach(&mut self, device: &str) {
        let request_string = serde_json::to_string(&Request::attach(device)).unwrap();
        self.send_text(&request_string).await;
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    pub async fn info(&mut self) -> Result {
        let request_string = serde_json::to_string(&Request::info()).unwrap();
        self.send_text(&request_string).await;

        match self.websocket.receive().await {
            Ok(res) => self.deserialize_response(res),
            Err(e) => panic!("{:#?}", e),
        }
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    pub async fn get_address(&mut self, offset: usize, length: usize) -> Result {
        let request_string = serde_json::to_string(&Request::get_address(offset, length)).unwrap();
        self.send_text(&request_string).await;

        match self.websocket.receive().await {
            Ok(res) => self.deserialize_response(res),
            Err(e) => panic!("{:#?}", e),
        }
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    async fn send_text(&mut self, request_string: &str) {
        println!("Request: {:#?}", request_string);
        if let Err(e) = self.websocket.send_text(request_string.into()).await {
            panic!("{:#?}", e);
        }
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    #[allow(clippy::unused_self)]
    fn deserialize_response(&self, frame: Frame) -> Result {
        println!("Response frame: {:#?}", frame);
        match frame {
            Frame::Text { payload, continuation: false, fin: true } => {
                match serde_json::from_str::<Result>(&payload) {
                    Ok(r) => r,
                    Err(e) => panic!("{:#?}", e),
                }
            }
            Frame::Binary { payload, continuation: false, fin: true } => {
                Result {
                    results: ResultData::Binary(payload),
                }
            }
            _ => panic!("Unhandled frame type"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
