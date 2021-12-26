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

// https://github.com/Skarsnik/QUsb2snes/blob/b06e818fd3ed4e5785dab48b22d13ff9160bf204/docs/Procotol.md
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
    pub async fn device_list(&mut self) -> std::result::Result<Vec<String>, Qusb2snesError> {
        let request_string = serde_json::to_string(&Request::device_list())?;
        self.send_text(&request_string).await?;

        if let ResultData::Text(result) = self.receive_until_fin().await? {
            return Ok(result);
        };

        Err(Qusb2snesError::FrameError { msg: "Unexpected response retrieving device list.".into() })
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
    pub async fn info(&mut self) -> std::result::Result<Vec<String>, Qusb2snesError> {
        let request_string = serde_json::to_string(&Request::info())?;
        self.send_text(&request_string).await?;

        if let ResultData::Text(result) = self.receive_until_fin().await? {
            return Ok(result);
        }

        Err(Qusb2snesError::FrameError { msg: "Unexpected response retrieving device info.".into() })
    }

    /// # Errors
    ///
    /// This method can fail if there is an issue sending the message to the associated
    /// [`websockets::WebSocket`], or if there is an issue deserializing the response
    /// from the websocket.
    pub async fn get_address(&mut self, offset: usize, length: usize) -> std::result::Result<Vec<u8>, Qusb2snesError> {
        let mut mem = vec![];

        for (start, len) in chunked_range(offset, length) {
            let request_string = serde_json::to_string(&Request::get_address(start, len))?;
            self.send_text(&request_string).await?;

            if let ResultData::Binary(res) = self.receive_until_fin().await? {
                mem.extend_from_slice(&res);
            } else {
                return Err(Qusb2snesError::FrameError { msg: "Unable to decode response".into() });
            };
        }

        Ok(mem)
    }

    /// # Panics
    ///
    /// Basically has no error handling yet.
    async fn send_text(&mut self, request_string: &str) -> std::result::Result<(), Qusb2snesError> {
        trace!("Request: {:?}", request_string);
        match self.websocket.send_text(request_string.into()).await.err() {
            Some(e) => Err(e.into()),
            None => Ok(()),
        }
    }

    async fn receive_until_fin(&mut self) -> std::result::Result<ResultData, Qusb2snesError> {
        let mut text_buf = vec![];
        let mut binary_buf = vec![];

        loop {
            let response = self.websocket.receive().await?;
            println!("Received: {:#?}", response);

            match response {
                Frame::Text { payload, fin, continuation: _ } => {
                    if let Result { results: ResultData::Text(res) } = serde_json::from_str::<Result>(&payload)?{
                        text_buf.extend_from_slice(&res);
                    } else {
                        return Err(Qusb2snesError::FrameError { msg: "Unable to handle text frame".into() });
                    };

                    if fin {
                        return Ok(ResultData::Text(text_buf));
                    }
                }
                Frame::Binary { payload, fin, continuation: _ } => {
                    binary_buf.extend_from_slice(&payload);
                    if fin {
                        return Ok(ResultData::Binary(binary_buf));
                    }
                }
                Frame::Close { payload: _ } => return Err(Qusb2snesError::FrameError { msg: "Websocket closed".into() }),
                _ => {}
            }
        }
    }
}

fn chunked_range(start: usize, length: usize) -> Vec<(usize, usize)> {
    let mut chunks = vec![];
    let page_size = 1_024;

    if length > page_size {
        let mut total_bytes = 0;
        while total_bytes < length {
            let current_page_size = std::cmp::min(page_size, length - total_bytes);
            chunks.push((start + total_bytes, current_page_size));
            total_bytes += current_page_size;
        }
    } else {
        chunks.push((start, length));
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn range_chunks() {
        assert_eq!(
            chunked_range(0, 10),
            vec![(0, 10)],
        );

        assert_eq!(
            chunked_range(0, 1024),
            vec![(0, 1024)],
        );

        assert_eq!(
            chunked_range(0, 2000),
            vec![(0, 1024), (1024, 976)],
        );

        assert_eq!(
            chunked_range(0xF5_0000, 0x2000),
            vec![
                (0xF5_0000, 1024),
                (0xF5_0400, 1024),
                (0xF5_0800, 1024),
                (0xF5_0C00, 1024),
                (0xF5_1000, 1024),
                (0xF5_1400, 1024),
                (0xF5_1800, 1024),
                (0xF5_1C00, 1024),
            ],
        );

        assert_eq!(
            chunked_range(0xF5_0000, 0x1FFF),
            vec![
                (0xF5_0000, 1024),
                (0xF5_0400, 1024),
                (0xF5_0800, 1024),
                (0xF5_0C00, 1024),
                (0xF5_1000, 1024),
                (0xF5_1400, 1024),
                (0xF5_1800, 1024),
                (0xF5_1C00, 1023),
            ],
        );
    }
}
