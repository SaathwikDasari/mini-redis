use bytes::BytesMut;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use crate::frame::Frame;
use std::fmt::{Error, format};
use std::io::Cursor;
use bytes::Buf;

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut
}

impl Connection {

    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(4096)
        }
    }

    /// - Ok(Some(Frame)): We successfully got a command.
    /// - Ok(None): The client disconnected cleanly.
    /// - Err(...): Something went wrong (network error).
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, String> {
        loop {
            // 1. Check if we already have a full frame parsed
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // 2. If not, read more data from the socket into the buffer
            // `read_buf` returns 0 if the connection is closed
            if 0 == self.stream.read_buf(&mut self.buffer).await.map_err(|e| e.to_string())? {
                // The remote closed the connection.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(" the connection was reset by peer".into());
                }
            };

        }
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>, String> {
        
            // Create the cursor
        let mut curs = Cursor::new(&self.buffer[..]);

        // Call our new standalone helper
        match parse(&mut curs)? {
            Some(frame) => {
                // If success, advance the REAL buffer by however much the cursor moved
                let len = curs.position() as usize;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            None => Ok(None),
        }
    }

}

fn parse(src: &mut Cursor<&[u8]>) -> Result<Option<Frame>, String> {
    if !src.has_remaining() {
        return Ok(None);
    }

    // 1. Peek the first byte
    let first_byte = src.get_u8();

    match first_byte {
        // Simple String (+)
        b'+' => {
            if let Some(line) = get_line(src)? {
                let string = String::from_utf8(line.to_vec())
                    .map_err(|_| "Invalid UTF-8".to_string())?;
                return Ok(Some(Frame::Simple(string)));
            }
            Ok(None)
        }
        // Bulk String ($)
        b'$' => {
            if let Some(line) = get_line(src)? {
                let len_str = String::from_utf8(line.to_vec())
                    .map_err(|_| "Invalid UTF-8 length".to_string())?;
                
                if len_str == "-1" {
                    return Ok(Some(Frame::Null));
                }

                let len: usize = len_str.parse().map_err(|_| "Invalid length")?;
                let n = len + 2; // +2 for \r\n

                if src.remaining() < n {
                    return Ok(None);
                }

                let data_start = src.position() as usize;
                let data_end = data_start + len;
                let data = src.get_ref()[data_start..data_end].to_vec();

                // Advance past the data + \r\n
                src.set_position((data_start + n) as u64);

                return Ok(Some(Frame::Bulk(data.into())));
            }
            Ok(None)
        }
        // NEW: Arrays (*)
        b'*' => {
            // 1. Read the length line: "*3\r\n" -> 3
            if let Some(line) = get_line(src)? {
                let len_str = String::from_utf8(line.to_vec())
                    .map_err(|_| "Invalid UTF-8 length".to_string())?;
                
                let len: usize = len_str.parse().map_err(|_| "Invalid array length")?;
                
                // 2. Loop `len` times and parse sub-frames
                let mut out = Vec::with_capacity(len);
                
                for _ in 0..len {
                    // RECURSION: Call this same function for the inner items
                    match parse(src)? {
                        Some(frame) => out.push(frame),
                        None => return Ok(None), // One of the inner items isn't ready yet
                    }
                }
                
                return Ok(Some(Frame::Array(out)));
            }
            Ok(None)
        }
        _ => Err(format!("protocol error; invalid frame type byte `{}`", first_byte)),
    }
}

// NOTE: You also need to move the `get_line` function OUT of `impl Connection`
// so this standalone function can see it. 
// Just cut and paste `get_line` to the bottom of the file too, removing `&self`.
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<Option<&'a [u8]>, String> {
    let start = src.position() as usize;
    let end = src.get_ref().len();

    for i in start..end - 1 {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(Some(&src.get_ref()[start..i]));
        }
    }
    Ok(None)
}