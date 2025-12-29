use bytes::BytesMut;
use tokio::net::TcpStream;
use crate::frame::Frame;
use std::io::Cursor;
use bytes::Buf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, String> {
        loop {
            // 1. Check if we already have a full frame parsed
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await.map_err(|e| e.to_string())? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(" the connection was reset by peer".into());
                }
            };

        }
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>, String> {
        
        let mut curs = Cursor::new(&self.buffer[..]);

        match parse(&mut curs)? {
            Some(frame) => {
                let len = curs.position() as usize;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            None => Ok(None),
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), std::io::Error> {
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
                self.stream.write_all(val.to_string().as_bytes()).await?; // Convert int to string
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                self.stream.write_u8(b'$').await?;
                self.stream.write_all(val.len().to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other, 
                    "Sending Arrays not supported in this mini-version"
                ));
            }
        }
        
        self.stream.flush().await?;
        
        Ok(())
    }

}

fn parse(src: &mut Cursor<&[u8]>) -> Result<Option<Frame>, String> {
    if !src.has_remaining() {
        return Ok(None);
    }

    let first_byte = src.get_u8();

    match first_byte {
        b'+' => {
            if let Some(line) = get_line(src)? {
                let string = String::from_utf8(line.to_vec())
                    .map_err(|_| "Invalid UTF-8".to_string())?;
                return Ok(Some(Frame::Simple(string)));
            }
            Ok(None)
        }
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
        b'*' => {
            if let Some(line) = get_line(src)? {
                let len_str = String::from_utf8(line.to_vec())
                    .map_err(|_| "Invalid UTF-8 length".to_string())?;
                
                let len: usize = len_str.parse().map_err(|_| "Invalid array length")?;
                
                let mut out = Vec::with_capacity(len);
                
                for _ in 0..len {
                    match parse(src)? {
                        Some(frame) => out.push(frame),
                        None => return Ok(None), 
                    }
                }
                
                return Ok(Some(Frame::Array(out)));
            }
            Ok(None)
        }
        _ => Err(format!("protocol error; invalid frame type byte `{}`", first_byte)),
    }
}

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