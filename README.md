# udp-sftp
Simple File Transfer Protocol (sFTP) and Ping implementation in Rust

Ping example:
```
cargo build
target/debug/ping_server 127.0.0.1 8081
# In new terminal
target/debug/ping_client 127.0.0.1 8081
```
sFTP example:
```
target/debug/sftp_server 127.0.0.1
# In new terminal
target/debug/sftp_client 127.0.0.1 input.txt
```
