use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use std::net::SocketAddr;
use std::net::UdpSocket;
mod serialization;
use serialization::{ByteOrder, Deserializer, Serializer, Value};

mod flight_models;
pub use flight_models::{Flight, Request, Response, FlightUpdate, MonitoringClient};

/// FlightController manages all flight-related operations and client monitoring
pub struct FlightController {
    /// Stores all flights, indexed by their flight ID
    pub flights: HashMap<i32, Flight>,
    /// Stores monitoring clients for each flight, indexed by flight ID
    monitoring_clients: HashMap<i32, HashSet<MonitoringClient>>,
}

impl FlightController {
    /// Creates a new FlightController instance
    pub fn new() -> Self {
        Self {
            flights: HashMap::new(),
            monitoring_clients: HashMap::new(),
        }
    }

    /// Handles incoming client requests and returns appropriate responses
    pub fn handle_request(&mut self, request: Request, socket: &UdpSocket, client_addr: Option<std::net::SocketAddr>) -> Response {
        // Clean expired monitors at the beginning of each request
        self.clean_expired_monitors();

        match request {
            Request::QueryFlightIds { source, destination } => {
                let ids = self.query_flight_ids(&source, &destination);
                if ids.is_empty() {
                    Response::Error("No matching flights found".to_string())
                } else {
                    Response::FlightIds(ids)
                }
            }
            Request::QueryFlightDetails { flight_id } => {
                if let Some(flight) = self.flights.get(&flight_id) {
                    Response::FlightDetails {
                        departure_time: Some(flight.departure_time),
                        airfare: Some(flight.airfare),
                        seats_available: Some(flight.seats_available),
                    }
                } else {
                    Response::Error("Flight not found".to_string())
                }
            }
            Request::ReserveSeats { flight_id, seats } => {
                let result = self.reserve_seats(flight_id, seats);
                match result {
                    Ok(_) => {
                        let updates = self.prepare_monitoring_updates(flight_id);
                        if !updates.is_empty() {
                            println!("Callback Triggered {:?}", updates);
                        }

                        // Send updates to monitoring clients
                        for (client_addr, update) in updates {
                            if update.flight_id == flight_id && seats > 0 {
                                println!("Sending Update to {:?}", client_addr);

                                // Serialize the update data
                                let mut serializer = Serializer::new(ByteOrder::Little);
                                let mut map = HashMap::new();
                                map.insert("action".to_string(), "5".to_string());
                                map.insert("flight_id".to_string(), flight_id.to_string());
                                map.insert("seats_available".to_string(), update.seats_available.to_string());
                                serializer.serialize_map(&map).unwrap();
                                let serialized_data = serializer.get_buffer();

                                // Send the serialized data to the client
                                socket.send_to(&serialized_data, client_addr).unwrap();
                            }
                        }
                        Response::Reservation(Ok(()))
                    }
                    Err(e) => Response::Reservation(Err(e))
                }
            }
            Request::MonitorFlight { flight_id, monitor_interval } => {
                let monitor_result = self.start_monitoring(flight_id, monitor_interval.try_into().unwrap(), client_addr.unwrap());
                match monitor_result {
                    Ok(_) => Response::MonitoringStarted(Ok(())),
                    Err(e) => Response::MonitoringStarted(Err(e))
                }
            }
        }
    }

    /// Queries flight IDs based on source and destination
    fn query_flight_ids(&self, source: &str, destination: &str) -> Vec<i32> {
        self.flights
            .iter()
            .filter(|(_, flight)| flight.source == source && flight.destination == destination)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Reserves seats for a given flight
    fn reserve_seats(&mut self, flight_id: i32, seats: i32) -> Result<(), String> {
        if let Some(flight) = self.flights.get_mut(&flight_id) {
            if flight.seats_available >= seats {
                flight.seats_available -= seats;
                Ok(())
            } else {
                Err("Not enough seats available".to_string())
            }
        } else {
            Err("Flight not found".to_string())
        }
    }
    
    /// Starts monitoring a flight for a client
    fn start_monitoring(&mut self, flight_id: i32, monitor_interval: i32, client_addr: std::net::SocketAddr) -> Result<(), String> {
        if self.flights.contains_key(&flight_id) {
            let expiration_time = Instant::now() + Duration::from_secs(monitor_interval as u64);
            let client = MonitoringClient {
                addr: client_addr,
                expiration_time,
            };
            self.monitoring_clients
                .entry(flight_id)
                .or_insert_with(HashSet::new)
                .insert(client);
            println!("Monitoring Clients {:?}", self.monitoring_clients);
            Ok(())
        } else {
            Err("Flight not found".to_string())
        }
    }

    /// Prepares updates for monitoring clients of a specific flight
    fn prepare_monitoring_updates(&self, flight_id: i32) -> Vec<(std::net::SocketAddr, FlightUpdate)> {
        let mut updates = Vec::new();
        if let Some(clients) = self.monitoring_clients.get(&flight_id) {
            if let Some(flight) = self.flights.get(&flight_id) {
                let update = FlightUpdate {
                    flight_id,
                    seats_available: flight.seats_available,
                };
                for client in clients {
                    updates.push((client.addr, update.clone()));
                }
            }
        }
        updates
    }

    /// Removes expired monitoring clients
    fn clean_expired_monitors(&mut self) {
        let now = Instant::now();
        for clients in self.monitoring_clients.values_mut() {
            clients.retain(|client| client.expiration_time > now);
        }
        self.monitoring_clients.retain(|_, clients| !clients.is_empty());
    }

    /// Returns a reference to the flights HashMap
    pub fn flights(&self) -> &HashMap<i32, Flight> {
        &self.flights
    }

    /// Adds a new flight to the controller
    pub fn add_flight(&mut self, flight: Flight) {
        self.flights.insert(flight.flight_id, flight);
    }

    // Commented out as it's not currently used
    // /// Queries details for a specific flight
    // fn query_flight_details(&self, flight_id: i32) -> Option<&Flight> {
    //     self.flights.get(&flight_id)
    // }
}