use std::io::{self, Write};
use std::net::UdpSocket;
use std::str;

// 导入配置模块
#[path = "../config.rs"]
mod config;
use config::Config;

fn main() -> io::Result<()> {
    // 加载配置
    let config = Config::load().expect("Failed to load config");
    
    // 创建UDP socket并绑定到任意可用端口
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    
    // 连接到服务器地址
    socket.connect(&config.server.address)?;

    loop {
        // 读取用户输入
        let mut input = String::new();
        print!("Enter message (or 'quit' to exit): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;

        let message = input.trim();
        if message == "quit" {
            break;
        }

        // 发送消息到服务器
        socket.send(message.as_bytes())?;

        // 接收服务器响应
        let mut buffer = [0u8; 1024];
        let (amt, _) = socket.recv_from(&mut buffer)?;
        
        // 将接收到的字节转换为字符串并打印
        let received = str::from_utf8(&buffer[..amt]).unwrap();
        println!("Received: {}", received);
    }

    Ok(())
}