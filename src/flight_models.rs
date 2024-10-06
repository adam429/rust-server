use std::net::SocketAddr;
use chrono::NaiveDateTime;

/// Represents a flight with its details
#[derive(Debug)]
pub struct Flight {
    pub flight_id: i32,        // Unique identifier for the flight
    pub source: String,        // Departure airport
    pub destination: String,   // Arrival airport
    pub departure_time: NaiveDateTime,  // Scheduled departure time
    pub airfare: f32,          // Price of the flight
    pub seats_available: i32,  // Number of available seats
}

/// Enum representing different types of requests that can be made to the flight system
#[derive(Debug)]
pub enum Request {
    /// Query to get flight IDs based on source and destination
    QueryFlightIds { 
        source: String,        // Departure airport
        destination: String    // Arrival airport
    },
    
    /// Query to get details of a specific flight
    QueryFlightDetails { 
        flight_id: i32         // ID of the flight to query
    },
    
    /// Request to reserve seats on a flight
    ReserveSeats { 
        flight_id: i32,        // ID of the flight to reserve seats on
        seats: i32             // Number of seats to reserve
    },
    
    /// Request to monitor updates for a specific flight
    MonitorFlight { 
        flight_id: i32,        // ID of the flight to monitor
        monitor_interval: i32  // Interval (in seconds) for monitoring updates
    },
}

/// Enum representing different types of responses from the flight system
#[derive(Debug)]
#[allow(dead_code)]
pub enum Response {
    /// Response containing a list of flight IDs
    FlightIds(Vec<i32>),
    
    /// Response containing details of a specific flight
    FlightDetails {
        departure_time: Option<NaiveDateTime>,  // Scheduled departure time (if available)
        airfare: Option<f32>,                   // Price of the flight (if available)
        seats_available: Option<i32>,           // Number of available seats (if available)
    },
    
    /// Response to a seat reservation request
    Reservation(Result<(), String>),  // Ok(()) if successful, Err(String) if failed
    
    /// Response to a flight monitoring request
    MonitoringStarted(Result<(), String>),  // Ok(()) if started successfully, Err(String) if failed
    
    /// General error response
    Error(String),  // Description of the error
}

/// Represents an update to a flight's information
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct FlightUpdate {
    pub flight_id: i32,        // ID of the flight that was updated
    pub seats_available: i32,  // New number of available seats
}

/// Represents a client that is monitoring flight updates
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct MonitoringClient {
    pub addr: SocketAddr,                  // Network address of the client
    pub expiration_time: std::time::Instant,  // Time when the monitoring should expire
}