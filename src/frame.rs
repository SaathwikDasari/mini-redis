use bytes::Bytes;


// This enum represents the 5 types of data Redis can send/receive
#[derive(Clone, Debug)]
pub enum Frame{
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>)
}