use std::io::{self, Write};
use std::net::UdpSocket;
use std::str;

#[path = "../config.rs"]
mod config;
use config::Config;

fn main() -> io::Result<()> {
    let config = Config::load().expect("Failed to load config");
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(&config.server.address)?;

    loop {
        let mut input = String::new();
        print!("Enter message (or 'quit' to exit): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;

        let message = input.trim();
        if message == "quit" {
            break;
        }

        socket.send(message.as_bytes())?;

        let mut buffer = [0u8; 1024];
        let (amt, _) = socket.recv_from(&mut buffer)?;
        
        let received = str::from_utf8(&buffer[..amt]).unwrap();
        println!("Received: {}", received);
    }

    Ok(())
}