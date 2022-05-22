use rand::{
  distributions::{Distribution, Uniform},
  Rng,
};
use std::{env, fs::File, io, net::SocketAddr, os::unix::fs::FileExt};
use tokio::{
  net::UdpSocket,
  time::{sleep, Duration},
};

#[tokio::main]
async fn main() -> io::Result<()> {
  //Parse input and create socket address
  let args: Vec<String> = env::args().collect();
  if args.len() != 2 {
    panic!("Usage: sftp_server <server_ipaddr>");
  }

  let server_ipaddr = &args[1];
  let server_port = "9093";
  let server_addr: SocketAddr = format!("{}:{}", server_ipaddr, server_port)
    .parse()
    .expect("Invalid ip address");

  //Adjust for simulated loss and delay on the system
  let loss_rate: f32 = 0.0;
  let average_delay: f32 = 100.0;

  //Create a uniform distribution between 0 and 1 for sampling
  let interval = Uniform::from(0f32..1f32);
  let mut rng = rand::thread_rng();

  //Bind server to socket with given address
  let sock = UdpSocket::bind(server_addr).await?;

  let mut recv_buf = [0; 513];
  let mut send_buf = [1; 1];

  //Open output file for writing
  let file = File::create("outputfile")?;

  //Holds previous sequence number to check for duplicates
  let mut sequence_number = 1;
  let mut byte_counter: u64 = 0;

  println!("Connection established at {}\n", server_addr); 

  loop {
    let rand = interval.sample(&mut rng) as f32;

    let (_len, addr) = sock.recv_from(&mut recv_buf).await?;

    //If duplicate not received, write data to outputfile and alternate ACK bit
    if recv_buf[0] != sequence_number {
      sequence_number = recv_buf[0];

      let bytes_written = file.write_at(&mut recv_buf[1..], byte_counter)?;

      byte_counter += bytes_written as u64;

      //End of file reached
      if bytes_written < 512 {
        break;
      }

      //Naive bit alternator
      send_buf[0] = match send_buf[0] {
        0 => 1,
        1 => 0,
        _ => 0,
      };

      if rand < loss_rate {
        continue;
      }

      let delay = rng.gen_range(0.0, (2 as f32) * average_delay) as f32;

      sleep(Duration::from_secs((delay / (1000 as f32)).round() as u64)).await;
    }

    sock.send_to(&send_buf[..], addr).await?;
  }

  println!("Shutting down");
  Ok(())
}
