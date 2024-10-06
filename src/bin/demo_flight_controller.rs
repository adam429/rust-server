use std::net::SocketAddr;
use std::net::UdpSocket;
use chrono::NaiveDateTime;

#[path = "../controller.rs"]
mod controller;
use controller::FlightController;


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let mut controller = FlightController::new();

    // Add some sample flights
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

    // Simulate client address
    let client_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    // Test QueryFlightIds
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


    // Test QueryFlightDetails
    let request = controller::Request::QueryFlightDetails { flight_id: 1 };
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("QueryFlightDetails (flight_id: 1) response: {:?}", response);

    // Test ReserveSeats
    let request = controller::Request::ReserveSeats { flight_id: 1, seats: 2 };
    let response = controller.handle_request(request, &socket, Some(client_addr) );
    println!("ReserveSeats (flight_id: 1, seats: 2) response: {:?}", response);

    // Test QueryFlightDetails
    let request = controller::Request::QueryFlightDetails { flight_id: 1 };
    let response = controller.handle_request(request, &socket, Some(client_addr) );
    println!("QueryFlightDetails (flight_id: 1) response: {:?}", response);

        // Test ReserveSeats
    let request = controller::Request::ReserveSeats { flight_id: 1, seats: 999 };
    let response = controller.handle_request(request, &socket, Some(client_addr)     );
    println!("ReserveSeats (flight_id: 1, seats: 999) response: {:?}", response);


    // Test MonitorFlight
    let request = controller::Request::MonitorFlight { flight_id: 1, monitor_interval: 1 };
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("MonitorFlight (flight_id: 1, monitor_interval: 60) response: {:?}", response);

        // Test ReserveSeats
    let request = controller::Request::ReserveSeats { flight_id: 1, seats: 2 };
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("ReserveSeats (flight_id: 1, seats: 2) response: {:?}", response);

    // delay 0.6 seconds
    std::thread::sleep(std::time::Duration::from_millis(600));

    // Test ReserveSeats
    let request = controller::Request::ReserveSeats { flight_id: 1, seats: 2 };
    let response = controller.handle_request(request, &socket, Some(client_addr));
    println!("ReserveSeats (flight_id: 1, seats: 2) response: {:?}", response);
    
    // delay 0.6 seconds
    std::thread::sleep(std::time::Duration::from_millis(600));

    // Test ReserveSeats
    let request = controller::Request::ReserveSeats { flight_id: 1, seats: 2 };
    let response = controller.handle_request(request, &socket, Some(client_addr) );
    println!("ReserveSeats (flight_id: 1, seats: 2) response: {:?}", response);
    


    // Print final state of flights
    println!("Final state of flights:");
    for (id, flight) in controller.flights() {
        println!("Flight {}: {:?}", id, flight);
    }
}