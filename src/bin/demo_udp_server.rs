use std::net::UdpSocket;
#[path = "../config.rs"]
mod config;
use config::Config;

fn main() -> std::io::Result<()> {
    // 加载配置文件
    let config = Config::load().expect("Failed to load config");
    
    // 绑定UDP socket到配置的地址
    let socket = UdpSocket::bind(&config.server.address)?;
    println!("UDP Echo Server listening on {}", config.server.address);

    // 创建一个缓冲区来存储接收到的数据
    let mut buf = [0; 1024];
    
    // 无限循环,持续监听incoming数据包
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                // 成功接收数据
                println!("Received {} bytes from {}", amt, src);
                
                // 准备要回显的消息
                let echo_message = &buf[..amt];
                
                // 将消息回显给发送者
                socket.send_to(echo_message, src)?;
                println!("Echoed message back to {}", src);
            }
            Err(e) => {
                // 接收数据时发生错误
                eprintln!("Couldn't receive a datagram: {}", e);
            }
        }
    }
}