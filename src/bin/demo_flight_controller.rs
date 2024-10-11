use std::net::SocketAddr;
use std::net::UdpSocket;
use chrono::NaiveDateTime;

#[path = "../controller.rs"]
mod controller;
use controller::FlightController;

fn main() {
    // 创建UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    // 初始化航班控制器
    let mut controller = FlightController::new();

    // 添加示例航班
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

    // 模拟客户端地址
    let client_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    // 测试QueryFlightIds功能
    let request = controller::Request::QueryFlightIds {
        source: "New York".to_string(),
        destination: "London".to_string(),
    };
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("QueryFlightIds (New York->London) response: {:?}", response);

    let request = controller::Request::QueryFlightIds {
        source: "London".to_string(),
        destination: "Paris".to_string(),
    };
    let response = controller.handle_request(request, &socket, Some(client_addr) );
    println!("QueryFlightIds (London->Paris) response: {:?}", response);

    // 测试QueryFlightDetails功能
    let request = controller::Request::QueryFlightDetails { flight_id: 1 };
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("QueryFlightDetails (flight_id: 1) response: {:?}", response);

    // 测试ReserveSeats功能
    let request = controller::Request::ReserveSeats { flight_id: 1, seats: 2 };
    let response = controller.handle_request(request, &socket, Some(client_addr) );
    println!("ReserveSeats (flight_id: 1, seats: 2) response: {:?}", response);

    // 测试MonitorFlight功能
    let request = controller::Request::MonitorFlight { flight_id: 1, monitor_interval: 1 };
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("MonitorFlight (flight_id: 1, monitor_interval: 60) response: {:?}", response);

    // 测试多次ReserveSeats以触发MonitorFlight通知
    for _ in 0..3 {
        let request = controller::Request::ReserveSeats { flight_id: 1, seats: 2 };
        let response = controller.handle_request(request, &socket, Some(client_addr));
        println!("ReserveSeats (flight_id: 1, seats: 2) response: {:?}", response);
        
        // 延迟0.6秒
        std::thread::sleep(std::time::Duration::from_millis(600));
    }

    // 打印航班的最终状态
    println!("Final state of flights:");
    for (id, flight) in controller.flights() {
        println!("Flight {}: {:?}", id, flight);
    }

    // 测试ReserveSeatsCheapestPrice功能
    let request = controller::Request::ReserveSeatsCheapestPrice {        
        source: "New York".to_string(),
        destination: "London".to_string(),
    };  
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("ReserveSeatsCheapestPrice (New York->London) response: {:?}", response);

    // 打印航班的更新状态
    println!("Final state of flights:");
    for (id, flight) in controller.flights() {
        println!("Flight {}: {:?}", id, flight);
    }

    // 测试ReserveSeatsBelowPrice功能
    let request = controller::Request::ReserveSeatsBelowPrice {        
        source: "New York".to_string(),
        destination: "London".to_string(),
        max_price: 600.0,
    };  
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("ReserveSeatsBelowPrice (New York->London) response: {:?}", response);

    // 打印航班的最终状态
    println!("Final state of flights:");
    for (id, flight) in controller.flights() {
        println!("Flight {}: {:?}", id, flight);
    }

    // 重置所有航班
    controller.reset_flights();

    // 打印重置后的航班状态
    println!("Final state of flights after reset:");
    for (id, flight) in controller.flights() {
        println!("Flight {}: {:?}", id, flight);
    }
}