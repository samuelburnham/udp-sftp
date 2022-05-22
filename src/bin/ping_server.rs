use rand::{
  distributions::{Distribution, Uniform},
  Rng,
};
use std::{env, io, net::SocketAddr, time};
use tokio::{net::UdpSocket, time::sleep};

#[tokio::main]
async fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();
  if args.len() != 3 {
    panic!("Usage: ping_server <host_name> <port_number>");
  }
  let host_name = &args[1];
  let port_number = &args[2];
  let server_addr: SocketAddr = format!("{}:{}", host_name, port_number)
    .parse()
    .expect("invalid host name and/or port number");

  let loss_rate: f32 = 0.3;
  let average_delay: f32 = 100.0;

  //Create a uniform distribution between 0 and 1 for sampling
  let interval = Uniform::from(0f32..1f32);
  let mut rng = rand::thread_rng();

  let sock = UdpSocket::bind(server_addr).await?;
  let mut buf = [0; 1024];

  println!("Connection established at {}\nWaiting for pings", server_addr);

  // Respond to incoming pings with simulated loss rate and delay
  loop {
    let rand = interval.sample(&mut rng) as f32;

    let (len, addr) = sock.recv_from(&mut buf).await?;
    println!("{} bytes received from {}", len, addr);

    // Sends or loses whole packet for simplicity
    if rand < loss_rate {
      println!("Reply not sent.\n");
      continue;
    }

    let delay = rng.gen_range(0.0, (2f32) * average_delay) as f32;

    sleep(time::Duration::from_millis(
      (delay / (1000f32)).round() as u64
    ))
    .await;

    let len = sock.send_to(&buf[..len], addr).await?;
    println!("{} bytes sent\n", len);
  }
}
