use std::collections::HashMap;
use std::io::{self, Write};
use std::net::UdpSocket;
use std::str;
use rand::Rng;
use chrono::NaiveDateTime;
use std::time::{Duration, Instant};

// 导入自定义模块
#[path = "../serialization.rs"]
mod serialization;
use serialization::{Serializer, Deserializer, ByteOrder};

#[path = "../controller.rs"]
mod controller;
use controller::{Request, Response};

#[path = "../config.rs"]
mod config;
use config::Config;

/// 生成随机的请求ID
fn gen_request_id() -> String {
    rand::thread_rng().gen_range(0..100000000).to_string()
}



fn send_request_and_receive_response(map: HashMap<String, String>, socket: &UdpSocket) -> Result<HashMap<String, String>, io::Error> {
    let config = Config::load().expect("Failed to load config");
    let retry = config.client.retry;
    let timeout = config.client.timeout;

    let mut serializer = Serializer::new(ByteOrder::Little);
    let timeout_duration = Duration::new(timeout.into(), 0); // 设置超时时间为10秒
    socket.set_read_timeout(Some(timeout_duration))?;
    let mut attempt = 0;

    // println!("request_id: {:?}", map.get("request_id").unwrap());
    // println!("timestamp: {:?}", chrono::Utc::now().timestamp());

    println!("Request: {:?}", map);

    serializer.serialize_map(&map)?;
    let send_buffer = serializer.get_buffer();
    socket.send(&send_buffer)?;

    let mut received_result = None;


    loop {
        let start_time = Instant::now();
        let mut buffer = [0u8; 1024];

        // 设置超时
        while start_time.elapsed() < timeout_duration {
            match socket.recv_from(&mut buffer) {
                Ok((amt, _)) => {
                    let received = &buffer[..amt];
                    let mut deserializer = Deserializer::new(received, ByteOrder::Little);
                    let value = deserializer.deserialize_next().unwrap();
                    let result: HashMap<String, String> = value.as_map().unwrap().iter()
                        .map(|(k, v)| (k.to_string(), v.as_string().unwrap().to_string()))
                        .collect();

                    println!("Received: {:?}", result);
                    received_result = Some(result);
                    break; // 成功接收到响应，退出循环
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // 如果没有数据可用，继续等待
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if received_result.is_some() {
            break; // 收到响应，退出尝试循环
        } else {
            attempt += 1;
            if attempt < retry {
                println!("No response received, resending request...");
                socket.send(&send_buffer)?; // 重新发送请求
            }
        }
    }

    received_result.ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "No response received after 2 attempts"))
}

/// 发送请求并处理响应
fn send_request(request: Request, socket: &UdpSocket) -> Result<Response, io::Error> {
    let request_id = gen_request_id();
    let mut map = HashMap::new();

    let config = Config::load().expect("Failed to load config");
    let invocation_semantic = config.client.invocation_semantic;

    println!("----------------------------------");
    match request {
        Request::QueryFlightIds { source, destination } => {
            // 构建查询航班ID的请求
            map.insert("request_id".to_string(), request_id);
            map.insert("invocation_semantic".to_string(), invocation_semantic);
            map.insert("action".to_string(), 1.to_string());
            map.insert("source".to_string(), source);
            map.insert("destination".to_string(), destination);

            // 序列化并发送请求
            let result = send_request_and_receive_response(map, socket).unwrap();

            // 处理响应数据
            if result.get("flight_ids").is_none() {
                Ok(Response::FlightIds(vec![]))
            } else {
                let flight_ids = result.get("flight_ids").unwrap()
                    .split(",").map(|s| s.parse().unwrap()).collect();
                Ok(Response::FlightIds(flight_ids))
            }
        }
        Request::QueryFlightDetails { flight_id } => {
            // 构建查询航班详情的请求
            map.insert("request_id".to_string(), request_id);
            map.insert("invocation_semantic".to_string(), invocation_semantic);
            map.insert("action".to_string(), 2.to_string());
            map.insert("flight_id".to_string(), flight_id.to_string());

            // 序列化并发送请求
            let result = send_request_and_receive_response(map, socket).unwrap();

            // 处理响应数据
            let status = result.get("status").unwrap();
            if status == "200" {
                let departure_time = NaiveDateTime::parse_from_str(
                    result.get("departure_time").unwrap(),
                    "%Y-%m-%d %H:%M:%S"
                ).unwrap();
                let airfare: f32 = result.get("airfare").unwrap().parse().unwrap();
                let seats_available: i32 = result.get("seats_available").unwrap().parse().unwrap();
                Ok(Response::FlightDetails {
                    departure_time: Some(departure_time),
                    airfare: Some(airfare),
                    seats_available: Some(seats_available)
                })
            } else {
                Ok(Response::FlightDetails {
                    departure_time: None,
                    airfare: None,
                    seats_available: None
                })
            }
        }
        Request::ReserveSeats { flight_id, seats } => {
            // 构建预订座位的请求
            map.insert("request_id".to_string(), request_id);
            map.insert("invocation_semantic".to_string(), invocation_semantic);
            map.insert("action".to_string(), 3.to_string());
            map.insert("flight_id".to_string(), flight_id.to_string());
            map.insert("seats".to_string(), seats.to_string());

            // 序列化并发送请求
            let result = send_request_and_receive_response(map, socket).unwrap();

            // 处理响应数据
            let status = result.get("status").unwrap();
            if status == "200" {
                Ok(Response::Reservation(Ok(())))
            } else {
                Ok(Response::Reservation(Err(result.get("message").unwrap().to_owned())))
            }
        }
        Request::MonitorFlight { flight_id, monitor_interval } => {
            // 构建监控航班的请求
            map.insert("request_id".to_string(), request_id);
            map.insert("invocation_semantic".to_string(), invocation_semantic);
            map.insert("action".to_string(), 4.to_string());
            map.insert("flight_id".to_string(), flight_id.to_string());
            map.insert("monitor_interval".to_string(), monitor_interval.to_string());

            // 序列化并发送请求
            let result = send_request_and_receive_response(map, socket).unwrap();

            // 处理响应数据
            let status = result.get("status").unwrap();
            if status == "200" {
                Ok(Response::MonitoringStarted(Ok(())))
            } else {
                Ok(Response::MonitoringStarted(Err(result.get("message").unwrap().to_owned())))
            }
        }
    }
}

fn main() -> io::Result<()> {
    // 加载配置并创建UDP socket
    let config = Config::load().expect("Failed to load config");
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    
    // println!("Local address: {:?}", socket.local_addr()?);
    println!("Server address: {:?}", &config.server.address);
    
    socket.connect(&config.server.address)?;

    // 主循环，处理用户输入和请求
    loop {
        let mut input = String::new();
        println!("----------------------------------");
        println!("Command List:");
        println!("  quit - exit the program");
        println!("  1 - query flight ids");
        println!("  2 - query flight details");
        println!("  3 - reserve seats");
        println!("  4 - monitor flight");
        print!("Enter command: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;

        let message = input.trim();
        if message == "quit" {
            break;
        } else if message == "1" {
            // 查询航班ID
            let mut input2 = String::new();
            print!("Enter source: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input2)?;
            let mut input3 = String::new();
            let source = input2.trim();
            print!("Enter destination: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input3)?;
            let destination = input3.trim();
            let request = Request::QueryFlightIds {
                source: source.to_string(),
                destination: destination.to_string(),
            };
            let response = send_request(request, &socket)?;
            println!("Result: {:?}", response);
        } else if message == "2" {
            // 查询航班详情
            let mut input2 = String::new();
            print!("Enter flight id: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input2)?;
            let flight_id = input2.trim();
            let request = Request::QueryFlightDetails {
                flight_id: flight_id.parse().unwrap(),
            };
            let response = send_request(request, &socket)?;
            println!("Result: {:?}", response);
        } else if message == "3" {
            // 预订座位
            let mut input2 = String::new();
            print!("Enter flight id: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input2)?;
            let flight_id = input2.trim();
            let mut input3 = String::new();
            print!("Enter seats: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input3)?;
            let seats = input3.trim();
            let request = Request::ReserveSeats {
                flight_id: flight_id.parse().unwrap(),
                seats: seats.parse().unwrap(),
            };
            let response = send_request(request, &socket)?;
            println!("Result: {:?}", response);
        } else if message == "4" {
            // 监控航班
            let mut input2 = String::new();
            print!("Enter flight id: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input2)?;
            let flight_id = input2.trim();
            let mut input3 = String::new();
            print!("Enter monitor_interval: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input3)?;
            let monitor_interval = input3.trim();
            let request = Request::MonitorFlight {
                flight_id: flight_id.parse().unwrap(),
                monitor_interval: monitor_interval.parse().unwrap(),
            };
            let response = send_request(request, &socket)?;
            println!("Result: {:?}", response);

            // 持续接收监控更新
            loop {
                println!("Waiting for monitor update...");
                let mut buffer = [0u8; 1024];
                let (amt, _) = socket.recv_from(&mut buffer)?;
                let received = &buffer[..amt];
                let mut deserializer = Deserializer::new(received, ByteOrder::Little);
                let value = deserializer.deserialize_next().unwrap();
                let result = value.as_map().unwrap();
                println!("Received: {:?}", result);
            }
        }
    }

    Ok(())
}