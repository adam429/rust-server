use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use chrono::NaiveDateTime;
use std::net::UdpSocket;
mod serialization;
use serialization::{ByteOrder,  Serializer};

mod flight_models;
pub use flight_models::{Flight, Request, Response, FlightUpdate, MonitoringClient};
use tracing_subscriber::filter;

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
                                tracing::info!("Sending Update to {:?}", client_addr);

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
            Request::ReserveSeatsCheapestPrice { source, destination } => {
                let result = self.reserve_seats_cheapest_price(&source, &destination);
                Response::ReserveSeatsCheapestPrice(result)
            }
            Request::ReserveSeatsBelowPrice { source, destination, max_price } => {
                let result = self.reserve_seats_below_price(&source, &destination, max_price);
                Response::ReserveSeatsBelowPrice(result)
            }
            Request::ResetFlights => {
                self.reset_flights();
                Response::ResetFlights(Ok(()))
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
            tracing::info!("Monitoring Clients {:?}", self.monitoring_clients);
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

    fn reserve_seats_cheapest_price(&mut self, source: &str, destination: &str) -> Result<(), String> {
        let flight_ids: Vec<i32> = self.flights
            .iter()
            .filter(|(_, flight)| flight.source == source && flight.destination == destination)
            .map(|(id, _)| *id)
            .collect();

        if flight_ids.is_empty() {
            return Err("No matching flights found".to_string());
        }

        let available_flight_ids: Vec<i32> = flight_ids.into_iter().filter(|id| self.flights[id].seats_available > 0).collect();

        if available_flight_ids.is_empty() {
            return Err("No seats available found".to_string());
        }

        let cheapest_flight_id = *available_flight_ids.iter().min_by_key(|id| (self.flights[*id].airfare *100.0) as i32).unwrap();

        // Reserve the seat for the cheapest flight
        if let Some(flight) = self.flights.get_mut(&cheapest_flight_id) {
            if flight.seats_available > 0 {
                flight.seats_available -= 1;
                Ok(())
            } else {
                return Err("No seats available found".to_string());
            }
        } else {
            Err("Flight not found".to_string())
        }
    }


    fn reserve_seats_below_price(&mut self, source: &str, destination: &str, max_price: f32) -> Result<(), String> {
        let flight_ids: Vec<i32> = self.flights
            .iter()
            .filter(|(_, flight)| flight.source == source && flight.destination == destination)
            .map(|(id, _)| *id)
            .collect();

        if flight_ids.is_empty() {
            return Err("No matching flights found".to_string());
        }

        let price_below_max_flight_ids: Vec<i32> = flight_ids.into_iter().filter(|id| self.flights[id].airfare <= max_price).collect();

        if price_below_max_flight_ids.is_empty() {
            return Err("No seats available found".to_string());
        }

        for id in price_below_max_flight_ids {
            if let Some(flight) = self.flights.get_mut(&id) {
                if flight.seats_available > 0 {
                    flight.seats_available = 0;
                }
            }
        }

        Ok(())
    }


    /// Returns a reference to the flights HashMap
    pub fn flights(&self) -> &HashMap<i32, Flight> {
        &self.flights
    }

    /// Adds a new flight to the controller
    pub fn add_flight(&mut self, flight: Flight) {
        self.flights.insert(flight.flight_id, flight);
    }

    pub fn remove_flight(&mut self, flight_id: i32) {
        self.flights.remove(&flight_id);
    }

    pub fn clear_flights(&mut self) {
        self.flights.clear();
    }


    pub fn reset_flights(&mut self) {
        self.flights.clear();
        self.monitoring_clients.clear();

        // 添加一些示例航班
        let flight0 = Flight {
            flight_id: 0,   
            source: "New York".to_string(),
            destination: "London".to_string(),
            departure_time: NaiveDateTime::parse_from_str("2024-08-30 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            airfare: 200.0,
            seats_available: 50,
        };
        self.add_flight(flight0);

        let flight1 = Flight {
            flight_id: 1,
            source: "New York".to_string(),
            destination: "London".to_string(),
            departure_time: NaiveDateTime::parse_from_str("2024-09-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            airfare: 500.0,
            seats_available: 100,
        };
        self.add_flight(flight1);

        let flight2 = Flight {
            flight_id: 2,
            source: "London".to_string(),
            destination: "Paris".to_string(),
            departure_time: NaiveDateTime::parse_from_str("2024-09-02 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            airfare: 300.0,
            seats_available: 150,
        };
        self.add_flight(flight2);        
                
    }

    // Commented out as it's not currently used
    // /// Queries details for a specific flight
    // fn query_flight_details(&self, flight_id: i32) -> Option<&Flight> {
    //     self.flights.get(&flight_id)
    // }
}