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

use thiserror::Error;
use tracing::trace;
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

#[derive(Error, Debug)]
pub enum Qusb2snesError {
    #[error("websocket error: {source}")]
    SocketError {
        #[from]
        source: websockets::WebSocketError,
    },
    #[error("unable to deserialize message: {source}")]
    MessageError{
        #[from]
        source: serde_json::error::Error,
    },
    #[error("unhandled message frame: {msg}")]
    FrameError {
        msg: String,
    },
}

impl Client {
    /// # Errors
    /// 
    /// Will return [`Err`] if the underlying [`websockets::WebSocket`] returns an [`Err`].
    pub async fn new() -> std::result::Result<Self, Qusb2snesError> {
        let websocket = WebSocket::connect("ws://localhost:8080").await?;

        Ok(Client { websocket })
    }

    /// # Errors
    /// 
    /// This method can fail if the underlying [`websockets::WebSocket`] has an error,
    /// or if there is an  issue deserializing the list of devices.
    pub async fn device_list(&mut self) -> std::result::Result<Result, Qusb2snesError> {
        let request_string = serde_json::to_string(&Request::device_list())?;
        self.send_text(&request_string).await?;

        let result = self.websocket.receive().await?;
        self.deserialize_response(result)
    }

    /// # Errors
    /// 
    /// This method can fail if the device name to attach to is not serializable as
    /// a JSON string.
    pub async fn attach(&mut self, device: &str) -> std::result::Result<(), Qusb2snesError> {
        let request_string = serde_json::to_string(&Request::attach(device))?;
        self.send_text(&request_string).await?;

        Ok(())
    }

    /// # Errors
    /// 
    /// This method can fail if there is an issue sending the message to the associated
    /// [`websockets::WebSocket`], or if there is an issue deserializing the response
    /// from the websocket.
    pub async fn info(&mut self) -> std::result::Result<Result, Qusb2snesError> {
        let request_string = serde_json::to_string(&Request::info())?;
        self.send_text(&request_string).await?;

        self.websocket.receive().await.map(|r| self.deserialize_response(r))?
    }

    /// # Errors
    /// 
    /// This method can fail if there is an issue sending the message to the associated
    /// [`websockets::WebSocket`], or if there is an issue deserializing the resposne
    /// from the websocket.
    pub async fn get_address(&mut self, offset: usize, length: usize) -> std::result::Result<Result, Qusb2snesError> {
        let request_string = serde_json::to_string(&Request::get_address(offset, length))?;
        self.send_text(&request_string).await?;

        self.websocket.receive().await.map(|r| self.deserialize_response(r))?
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    async fn send_text(&mut self, request_string: &str) -> std::result::Result<(), Qusb2snesError> {
        trace!("Request: {:#?}", request_string);
        match self.websocket.send_text(request_string.into()).await.err() {
            Some(e) => Err(e.into()),
            None => Ok(()),
        }
    }

    /// # Panics
    /// 
    /// Basically has no error handling yet.
    #[allow(clippy::unused_self)]
    fn deserialize_response(&self, frame: Frame) -> std::result::Result<Result, Qusb2snesError> {
        trace!("Response frame: {:#?}", frame);
        match frame {
            Frame::Text { payload, continuation: false, fin: true } => {
                Ok(serde_json::from_str::<Result>(&payload)?)
            }
            Frame::Binary { payload, continuation: false, fin: true } => {
                Ok(Result {
                    results: ResultData::Binary(payload),
                })
            }
            _ => Err(Qusb2snesError::FrameError { msg: "Unhandled frame type".into() }),
        }
    }
}
