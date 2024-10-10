use std::net::UdpSocket;
use std::net::SocketAddr;
use std::net::Ipv4Addr;
use std::error::Error;
use std::collections::HashMap;
use chrono::NaiveDateTime;
use chrono::Utc;
use std::sync::{Arc, Mutex};

// 导入配置模块
#[path = "../config.rs"]
mod config;
use config::Config;

// 导入控制器模块
#[path = "../controller.rs"]
mod controller;
use controller::{FlightController};

// 导入序列化模块
#[path = "../serialization.rs"]
mod serialization;
use serialization::{ByteOrder, Deserializer, Serializer, Value};

/// 初始化航班控制器并添加示例航班
fn init_flight_controller() -> FlightController {
    let mut controller = FlightController::new();

    // 添加一些示例航班
    let flight0 = controller::Flight {
        flight_id: 0,
        source: "New York".to_string(),
        destination: "London".to_string(),
        departure_time: NaiveDateTime::parse_from_str("2024-08-30 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        airfare: 200.0,
        seats_available: 50,
    };
    controller.add_flight(flight0);

    let flight1 = controller::Flight {
        flight_id: 1,
        source: "New York".to_string(),
        destination: "London".to_string(),
        departure_time: NaiveDateTime::parse_from_str("2024-09-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        airfare: 500.0,
        seats_available: 100,
    };
    controller.add_flight(flight1);

    let flight2 = controller::Flight {
        flight_id: 2,
        source: "London".to_string(),
        destination: "Paris".to_string(),
        departure_time: NaiveDateTime::parse_from_str("2024-09-02 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        airfare: 300.0,
        seats_available: 150,
    };
    controller.add_flight(flight2);

    controller
}

struct RequestInfo {
    timestamp: NaiveDateTime,
    response: Vec<u8>,
}

// 创建一个全局的store_request
lazy_static::lazy_static! {
    static ref STORE_REQUEST: Arc<Mutex<HashMap<String, RequestInfo>>> = Arc::new(Mutex::new(HashMap::new()));
}


/// 主函数：启动UDP服务器并处理客户端请求
fn main() -> Result<(), Box<dyn Error>> {
    // 加载配置
    let config = Config::load().expect("Failed to load config");
    // 初始化航班控制器
    let mut flight_controller = &mut init_flight_controller();
    // 绑定UDP socket
    let socket = UdpSocket::bind(&config.server.address)?;
    println!("UDP Server listening on {}", config.server.address);

    let mut buf = [0; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let request_data = &buf[..amt];

                let mut deserializer = Deserializer::new(request_data, ByteOrder::Little);
                let payload = deserializer.deserialize_next()?;
                let payload = payload.as_map().ok_or("Invalid payload format")?;

                let request_id = payload.get("request_id").unwrap().as_string().unwrap();
                let invocation_semantic = payload.get("invocation_semantic").unwrap().as_string().unwrap();
                println!("----------------------------------");
                println!("request_id: {}", request_id);
                println!("invocation_semantic: {}", invocation_semantic);
            
                if invocation_semantic == "at-least-once" {
                    // 处理客户端请求
                    match handle_request(request_data, flight_controller, src, &socket) {
                        Ok(response) => {
                            let loss_rate = config.server.loss_rate;
                            let random_number = rand::random::<f32>();

                            // 在发送响应之前，将响应存储到全局store_request中
                            let mut store = STORE_REQUEST.lock().unwrap();
                            store.insert(request_id.to_string(), RequestInfo {
                                timestamp: Utc::now().naive_utc(),
                                response: response.clone(),
                            });

                            println!("store len: {}", store.len());
                                                        
                            if random_number > loss_rate {
                                socket.send_to(&response, src)?;
                                println!("Sent response to {}", src);
                            } else {
                                println!("Loss Rate Triggered: Dropped response");
                            }

                        }
                        Err(e) => {
                            eprintln!("Error processing request: {}", e);
                        }
                    }
                }
                if invocation_semantic == "at-most-once" {

                    let store = STORE_REQUEST.lock().unwrap();
                    if let Some(info) = store.get(request_id) {
                        // 如果已经处理过，直接发送存储的响应
                        socket.send_to(&info.response, src)?;
                        println!("Sent cached response to {}", src);
                    } else {
                        // 如果是新请求，处理并存储响应
                        drop(store); // 释放锁
                        match handle_request(request_data, flight_controller, src, &socket) {
                            Ok(response) => {
                                let loss_rate = config.server.loss_rate;
                                let random_number = rand::random::<f32>();

                                let mut store = STORE_REQUEST.lock().unwrap();
                                store.insert(request_id.to_string(), RequestInfo {
                                    timestamp: Utc::now().naive_utc(),
                                    response: response.clone(),
                                });

                                if random_number > loss_rate {
                                    socket.send_to(&response, src)?;
                                    println!("Sent response to {}", src);
                                } else {
                                    println!("Loss Rate Triggered: Dropped response");
                                }
                            }
                            Err(e) => {
                                eprintln!("Error processing request: {}", e);
                            }
                        }
                    }

                }
            }
            Err(e) => {
                eprintln!("Couldn't receive a datagram: {}", e);
            }
        }
    }
}

/// 处理客户端请求
fn handle_request(data: &[u8],  mut controller: &mut FlightController, src: SocketAddr, socket: &UdpSocket) -> Result<Vec<u8>, Box<dyn Error>> {
    // 反序列化请求数据
    let mut deserializer = Deserializer::new(data, ByteOrder::Little);
    let payload = deserializer.deserialize_next()?;
    println!("----------------------------------");
    println!("{:?} Request: {:?}", src, payload);

    let client_addr = src.to_string();
    let payload = payload.as_map().ok_or("Invalid payload format")?;
    
    // 提取action和request_id
    let action = payload.get("action")
        .ok_or("Missing 'action' field")?
        .as_string()
        .ok_or("Invalid 'action' type")?;

    let request_id = payload.get("request_id")
        .ok_or("Missing 'request_id' field")?
        .as_string()
        .ok_or("Invalid 'request_id' type")?;

    // 根据action调用相应的处理函数
    let mut response = match action.as_str() {
        "1" => query_flight_ids(payload, controller, socket),
        "2" => query_flight_details(&payload, controller, socket),
        "3" => reserve_seats(payload, &mut controller, socket),
        "4" => monitor_flight(payload, &mut controller, src, socket),
        _ => Err("Invalid action".into()),
    }?;

    // 添加request_id到响应中
    response.insert("request_id".to_string(), request_id.to_string());

    println!("Response: {:?}", response);

    // 序列化响应数据
    let mut serializer = Serializer::new(ByteOrder::Little);
    serializer.serialize_map(&response)?;
    Ok(serializer.get_buffer())
}

/// 查询航班ID
fn query_flight_ids(payload: &HashMap<String, Value>, controller: &mut FlightController,  socket: &UdpSocket) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let source = payload.get("source").unwrap().as_string().unwrap();
    let destination = payload.get("destination").unwrap().as_string().unwrap();

    let request = controller::Request::QueryFlightIds { source: source.to_string(), destination: destination.to_string() };
    let response = controller.handle_request(request, &socket, None); 

    println!("response: {:?}", response);

    match response {
        controller::Response::FlightIds(flight_ids) => {
            if flight_ids.is_empty() {
                let mut data = HashMap::new();
                data.insert("status".to_string(), "500".to_string());
                data.insert("message".to_string(), "No matching flights found".to_string());
                Ok(data)
            } else {
                let flight_ids = flight_ids.iter().map(|&id| id.to_string()).collect::<Vec<_>>().join(",");
                let mut data = HashMap::new();
                data.insert("status".to_string(), "200".to_string());   
                data.insert("flight_ids".to_string(), flight_ids);
                Ok(data)
            } 
        }
        controller::Response::Error(e) => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), e);
            Ok(data)
        }
        _ => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), "Unknown error".to_string());
            Ok(data)
        }
    }
}

/// 查询航班详情
fn query_flight_details(payload: &HashMap<String, Value>, controller: &mut FlightController, socket: &UdpSocket) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let flight_id = payload.get("flight_id").unwrap().as_string().unwrap();

    let request = controller::Request::QueryFlightDetails { flight_id: flight_id.parse::<i32>().unwrap() };
    println!("request: {:?}", request);
    let response = controller.handle_request(request, &socket, None);
    println!("response: {:?}", response);

    match response {
        controller::Response::FlightDetails { departure_time, airfare, seats_available } => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "200".to_string());
            data.insert("departure_time".to_string(), departure_time.unwrap().to_string());
            data.insert("airfare".to_string(), airfare.unwrap().to_string());
            data.insert("seats_available".to_string(), seats_available.unwrap().to_string());
            Ok(data)
        }
        controller::Response::Error(e) => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), e);
            Ok(data)
        }
        _ => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), "Unknown error".to_string());
            Ok(data)
        }   
    }
}

/// 预订座位
fn reserve_seats(payload: &HashMap<String, Value>, controller: &mut FlightController, socket: &UdpSocket) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let flight_id = payload.get("flight_id").unwrap().as_string().unwrap();
    let seats = payload.get("seats").unwrap().as_string().unwrap();

    let request = controller::Request::ReserveSeats { flight_id: flight_id.parse::<i32>().unwrap(), seats: seats.parse::<i32>().unwrap() };
    println!("request: {:?}", request);
    let response = controller.handle_request(request, &socket, None);
    println!("response: {:?}", response);

    match response {
        controller::Response::Reservation(reservation_result) => {
            if reservation_result.is_err() {
                let mut data = HashMap::new();
                data.insert("status".to_string(), "500".to_string());
                data.insert("message".to_string(), reservation_result.err().unwrap());
                Ok(data)
            } else {
                let mut data = HashMap::new();
                data.insert("status".to_string(), "200".to_string());
                Ok(data)
            }
        }
        controller::Response::Error(e) => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), e);
            Ok(data)
        }
        _ => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), "Unknown error".to_string());
            Ok(data)
        }
    }
}

/// 监控航班
fn monitor_flight(payload: &HashMap<String, Value>, controller: &mut FlightController, client_addr: SocketAddr, socket: &UdpSocket) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let flight_id = payload.get("flight_id").unwrap().as_string().unwrap().parse::<i32>().unwrap();
    let monitor_interval = payload.get("monitor_interval").unwrap().as_string().unwrap().parse::<i32>().unwrap();

    let request = controller::Request::MonitorFlight { flight_id: flight_id, monitor_interval: monitor_interval };
    println!("request: {:?}", request);
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("response: {:?}", response);

    match response {
        controller::Response::MonitoringStarted(monitor_result) => {
            if monitor_result.is_err() {
                let mut data = HashMap::new();
                data.insert("status".to_string(), "500".to_string());
                data.insert("message".to_string(), monitor_result.err().unwrap());
                Ok(data)
            } else {
                let mut data = HashMap::new();  
                data.insert("status".to_string(), "200".to_string());
                Ok(data)
            }
        }
        controller::Response::Error(e) => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), e);
            Ok(data)
        }
        _ => {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "500".to_string());
            data.insert("message".to_string(), "Unknown error".to_string());
            Ok(data)
        }
    }
}