# Mini-Redis (Rust)

A simplified, asynchronous Redis server implementation built from scratch in Rust. This project demonstrates low-level systems programming concepts including TCP networking, protocol parsing, and thread-safe concurrency.

## 🚀 Features

* **Asynchronous Core:** Built on `tokio` for handling thousands of concurrent connections.
* **RESP Parser:** Custom recursive parser for the Redis Serialization Protocol (RESP).
* **Thread-Safe Storage:** Uses `Arc<Mutex<HashMap>>` for safe shared state across threads.
* **Zero-Copy Networking:** Leverages the `bytes` crate and `BytesMut` for efficient memory handling.

## 🛠️ Getting Started

### Prerequisites
* Rust and Cargo installed.
* (Optional) Telnet or Netcat for testing.

### Running the Server
```bash
cargo run
```


The server will start listening on 127.0.0.1:6379.

🧪 How to Test
Since this server speaks raw Redis protocol, you can test it using Telnet or Netcat.

1. Connect
Open a new terminal and run:

```bash
telnet 127.0.0.1 6379
# OR
nc 127.0.0.1 6379
```

2. Send Commands
You must type the raw RESP protocol manually.

To Set a Value (SET mykey hello):

```plaintext
*3
$3
SET
$5
mykey
$5
hello
```

```plaintext
*2
$3
GET
$5
mykey
```


Expected Response: $5\r\nhello\r\n

📂 Project Structure
src/main.rs: The entry point. Initializes the DB, listens for connections, and spawns async tasks.

src/connection.rs: Handles the TCP stream. Contains the buffer (BytesMut) and the logic to read/write frames.

src/frame.rs: Defines the Frame enum (Simple, Bulk, Array, etc.) representing Redis data types.

src/db.rs: The database core. Wraps a HashMap in an Arc<Mutex<...>> to allow safe access from multiple threads.

📚 Dependencies
tokio: Asynchronous runtime.

bytes: Utilities for working with bytes efficiently.