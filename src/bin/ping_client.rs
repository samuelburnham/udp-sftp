use chrono::Utc;
use std::{env, io, net::SocketAddr, time::Duration};
use tokio::{
  net::UdpSocket,
  time::{sleep, timeout},
};

#[tokio::main]
async fn main() -> io::Result<()> {
  //Parse input and create valid socket address in decimal notation
  let args: Vec<String> = env::args().collect();
  if args.len() != 3 {
    panic!("Usage: ping_client <host_name> <port_number>");
  }
  let host_name = &args[1];
  let port_number = &args[2];
  let server_addr: SocketAddr = format!("{}:{}", host_name, port_number)
    .parse()
    .expect("invalid host name and/or port number");

  //Bind the client to a socket with the same IP address and 1 + the server's port
  let bind_addr: SocketAddr = format!(
    "{}:{}",
    host_name,
    (port_number.parse::<u32>().expect("Invalid port number") + 1).to_string()
  )
  .parse()
  .expect("Invalid host name and/or port number");
  println!("Server addr: {}\nClient addr: {}\n", server_addr, bind_addr);

  let sock = UdpSocket::bind(bind_addr).await?;
  //Arbitrary message to send to server
  let mut buf = [b'a'; 56];
  let mut sequence_number = 0;

  // Send five pings to the given server
  for _ in 0..5 {
    sock.send_to(&buf[..], server_addr).await?;

    let timer_start = Utc::now();

    let sleep_time = match timeout(Duration::from_millis(1000), sock.recv_from(&mut buf)).await {
      Ok(_) => {
        let timer_val = (Utc::now() - timer_start).to_std().unwrap().as_secs_f32();
        println!(
          "PING {} {} {:.3}",
          server_addr.ip(),
          sequence_number,
          timer_val * 1000f32
        );
        sequence_number += 1;
        1f32 - timer_val
      }
      Err(_) => {
        println!("PING {} {} LOST", server_addr.ip(), sequence_number);
        0f32
      }
    };

    //Sleep until at least one second has elapsed for timely delivery of pings
    sleep(Duration::from_secs_f32(sleep_time)).await;
  }
  Ok(())
}
