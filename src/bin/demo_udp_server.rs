use std::net::UdpSocket;
#[path = "../config.rs"]
mod config;
use config::Config;

fn main() -> std::io::Result<()> {
    let config = Config::load().expect("Failed to load config");
    let socket = UdpSocket::bind(&config.server.address)?;
    println!("UDP Echo Server listening on {}", config.server.address);

    let mut buf = [0; 1024];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("Received {} bytes from {}", amt, src);
                let echo_message = &buf[..amt];
                socket.send_to(echo_message, src)?;
                println!("Echoed message back to {}", src);
            }
            Err(e) => {
                eprintln!("Couldn't receive a datagram: {}", e);
            }
        }
    }
}