# Flight Reservation System

This is a UDP-based flight reservation system implemented in Rust. The system consists of a server and a client, allowing users to query flight information, reserve seats, and monitor flight updates.

## Project Structure

The project is organized into the following main components:

1. Server (`src/bin/server.rs`)
2. Client (`src/bin/client.rs`)
3. Flight Controller (`src/controller.rs`)
4. Serialization (`src/serialization.rs`)
5. Configuration (`src/config.rs`)
6. Flight Models (`src/flight_models.rs`)

## Features

- Query flight IDs based on source and destination
- Query flight details
- Reserve seats on a flight
- Monitor flight updates

## Configuration

The server address is configured using a `config.toml` file. Create this file in the project root with the following content:

```toml
server = { address = "0.0.0.0:8080" }
```

## Running the Server

To start the server, run:

```bash
cargo run --bin server
```

## Running the Client

To start the client, run:

```bash
cargo run --bin client
```

