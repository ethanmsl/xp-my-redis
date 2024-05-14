//! Framing our bytestreams

use bytes::Buf;
use bytes::BytesMut;
use mini_redis::frame::Error::Incomplete;
use mini_redis::{Frame, Result};
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::BufWriter;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    /// Generate new Connection from a TcpStream
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            // Allocate the buffer with 4kb of capacity.
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// ...
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        // Attempt frame from buffered data.  Return if possible.
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            // Try to get more data.
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // Remote closed the connection.  Check if incomplete frame in buffer.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    /// ...
    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // Create `T: Buf` type
        let mut buf = Cursor::new(&self.buffer[..]);
        // Check if full frame available
        match Frame::check(&mut buf) {
            Ok(_) => {
                //Get the byte length of frame
                let len = buf.position() as usize;
                // Reset internal cursor
                buf.set_position(0);
                // Parse the frame
                let frame = Frame::parse(&mut buf)?;
                // Discard th frame from the buffer
                self.buffer.advance(len);

                Ok(Some(frame))
            }
            Err(Incomple) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// ...
    async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                // self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();

                self.stream.write_u8(b'$').await?;
                // self.write_decimal(len as u64).await?; // not implemented here
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!(),
        }

        self.stream.flush().await;
        Ok(())
    }
}
