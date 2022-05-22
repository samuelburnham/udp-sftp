use chrono::Utc;
use std::{env, fs::File, io, net::SocketAddr, os::unix::fs::FileExt, time::Duration};
use tokio::{net::UdpSocket, time::timeout};

#[tokio::main]
async fn main() -> io::Result<()> {
  //Parse input and create socket address for server and client
  let args: Vec<String> = env::args().collect();
  if args.len() != 2 {
    panic!("Usage: sftp_client <server_ipaddr>");
  }
  let server_ipaddr = &args[1];
  let server_port = "9093";
  let client_port = "8080";

  let server_addr: SocketAddr = format!("{}:{}", server_ipaddr, server_port)
    .parse()
    .expect("invalid host name and/or port number");

  //Bind the client to a socket with the same IP address as server and arbitrary port
  let client_addr: SocketAddr = format!("{}:{}", server_ipaddr, client_port)
    .parse()
    .expect("invalid host name and/or port number");

  println!(
    "Server addr: {}\nClient addr: {}\n",
    server_addr, client_addr
  );

  let sock = UdpSocket::bind(client_addr).await?;
  let mut buf = [0; 513];

  //Open file to read
  let file = File::open("inputfile")?;
  let mut byte_counter: u64 = 0;
  let mut eof = false;

  let retransmission_timer = 1;
  let retransmission_limit = 5;

  let timer_start = Utc::now();

  //Read file and send to server until retransmission failure or end of file reached
  while !eof {
    let bytes_read = file.read_at(&mut buf[1..], byte_counter)?;

    //End of file reached
    if bytes_read < 512 {
      eof = true;
      //Set outdated bits from previous read to zero
      for i in (bytes_read + 1)..513 {
        buf[i] = 0;
      }
    }

    for i in 0..retransmission_limit {
      sock.send_to(&buf[..], server_addr).await?;

      //Read for one second before timing out and trying again if possible
      match timeout(
        Duration::from_secs(retransmission_timer),
        sock.recv_from(&mut buf),
      )
      .await
      {
        Ok(_) => {
          //Naive bit alternator
          buf[0] = match buf[0] {
            0 => 1,
            1 => 0,
            _ => 0,
          };
          byte_counter += 512;
          break;
        }
        Err(_) => {
          if i == retransmission_limit - 1 {
            println!("sFTP: file transfer unsuccessful: packet retransmission limit reached");
            std::process::exit(1);
          }
        }
      };
    }
  }

  //Calculate elapsed time between beginning and end of succesful file transfer
  let rtt = (Utc::now() - timer_start).to_std().unwrap().as_secs_f32();
  println!(
    "sFTP: file sent successfully to {} in {:.3} secs",
    server_addr.ip(),
    rtt
  );

  Ok(())
}
