//! Keeps track of all connected clients and a shared state.

use std::collections::HashMap;
use std::time::Duration;

use actix::prelude::{
    Actor, ActorFuture, Context, Handler, Recipient, ResponseActFuture, WrapFuture,
};
use actix::AsyncContext;
use mongodb::bson::doc;
use rand::Rng;
use uuid::Uuid;

use crate::client::ClientData;
use crate::config::{Config, CONFIG_FILE_PATH};
use crate::database::DbConnection;
use crate::gtfs::trafiklab::TrafiklabApi;
use crate::messages::{
    Connect, Disconnect, EchoPositions, PassengerInfo, PositionUpdate, ReserveSeat, RouteRequest,
    UnreserveSeat, WsMessage,
};
use crate::protocol::server_protocol::{
    ErrorType, PassengerInformationOutput, RouteInformationOutput, ServerOutput, Vehicle,
    VehiclePositionsOutput,
};
use crate::util::{filter_vehicle_position, only_numbers};

/// Type alias, which is essentially an address to an actor which you can
/// send messages to.
pub type Socket = Recipient<WsMessage>;

/// The lobby keeps track of a common/shared state between all clients.
pub struct Lobby {
    /// Maps client IDs to client data.
    clients: HashMap<Uuid, ClientData>,

    /// Handle to communicate with Trafiklab's API.
    trafiklab: TrafiklabApi,

    /// The interval in which data is fetched from the external Trafiklab API and
    /// echoed out to all connected clients.
    echo_positions_interval: Duration,

    /// Handle to a connection to a MongoDB database.
    db_connection: DbConnection,

    /// NOTE THAT THIS IS VERY TEMPORARY. THIS FUNCTIONALITY SHOULD BE MOVED
    /// TO AN EXTERNAL DATABASE IN THE FUTURE.
    /// Maps a vehicle descriptor id (string) a passenger information object.
    passenger_info: HashMap<String, PassengerInformationOutput>,

    /// Random number generator.
    rng: rand::rngs::ThreadRng,
}

impl Lobby {
    pub fn new(db_connection: DbConnection) -> Self {
        let mut config_handler = Config::new();

        // If the load somehow fails the program will panic since it cannot operate
        // without the necessary data.
        if let Err(reason) = config_handler.load_config(CONFIG_FILE_PATH) {
            panic!("{}", reason);
        }

        // Try to get the API keys from the parsed config. This program is supposed to panic
        // when one of these fail to retrieve a value, hence the unwrap call.
        let realtime_key = config_handler
            .get_trafiklab_value_str("realtime_key")
            .expect("realtime_key is missing from config file");
        let static_key = config_handler
            .get_trafiklab_value_str("static_key")
            .expect("static_key is missing from config file");
        let echo_interval: f64 = config_handler
            .get_trafiklab_value_f64("echo_interval")
            .expect("echo_interval is missing or not number in config file");

        let mut lobby = Lobby {
            clients: HashMap::new(),
            trafiklab: TrafiklabApi::new(realtime_key, static_key),
            echo_positions_interval: Duration::from_secs_f64(echo_interval),
            db_connection,
            passenger_info: HashMap::new(),
            rng: rand::thread_rng(),
        };

        // Fetch initial realtime data.
        lobby
            .trafiklab
            .fetch_vehicle_positions()
            .expect("Could not fetch realtime data from Trafiklab.");

        lobby
    }

    /// Returns POSIX timestamp in seconds since 1970-01-01 00:00:00.
    fn get_current_timestamp() -> u64 {
        let start = std::time::SystemTime::now();
        let since_epoch_start = start.duration_since(std::time::UNIX_EPOCH).unwrap();

        since_epoch_start.as_secs()
    }

    /// This method starts an interval which fetches new data from the Trafiklab API.
    fn start_echo_positions_interval(&mut self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(self.echo_positions_interval, |act, ctx| {
            // If no clients are connected there is no point in fetching any data.
            if act.clients.is_empty() {
                return;
            }

            // Fetch vehicle positions from Trafiklab's API.
            match act.trafiklab.fetch_vehicle_positions() {
                Err(reason) => {
                    println!(
                        "Failed to retrieve data from Trafiklab Realtime API. Reason: {}",
                        reason
                    );

                    // TODO: Send error message to clients indicating that the server cannot receive
                    // data from the external API.
                    return;
                }
                Ok(()) => (),
            }

            // Since we cannot handle asynchronous calls here, we defer to a message handler that
            // can handle asynchronous calls easily.
            ctx.address().do_send(EchoPositions);
        });
    }
}

impl Lobby {
    /// Sends a message to a specific client.
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(recipient) = self.clients.get(id_to) {
            let _ = recipient.addr.do_send(WsMessage(message.to_owned()));
        } else {
            println!("Attempting to send message but couldn't find client id.");
        }
    }

    /// Sends a message to every connected client stored in self.clients.
    #[allow(dead_code)]
    fn send_to_everyone(&self, message: &str) {
        self.clients
            .keys()
            .for_each(|client_id| self.send_message(message, client_id));
    }

    /// Sends a message to every connected client stored in self.clients.
    #[allow(dead_code)]
    fn send_to_everyone_except_self(&self, message: &str, self_id: &Uuid) {
        self.clients
            .keys()
            .filter(|client_id| *client_id.to_owned() != *self_id)
            .for_each(|client_id| self.send_message(message, client_id));
    }

    /// Sends an error message to a client.
    #[allow(dead_code)]
    fn send_error(&self, id_to: &Uuid, error_type: ErrorType, error_message: String) {
        self.send_message(
            &ServerOutput::error_message(error_type, error_message),
            id_to,
        );
    }

    /// Sends a status message to all connected clients that have an active reservation for a bus
    ///
    fn send_passenger_update(&self, descriptor_id: &str) {
        // Get the passenger info object.
        let passenger_info = self.passenger_info.get(descriptor_id).unwrap().clone();

        // Create the message that should be sent.
        let message =
            serde_json::to_string(&ServerOutput::PassengerInformation(passenger_info)).unwrap();

        self.clients.iter().for_each(|(id, client)| {
            if let Some(client_descriptor_id) = &client.last_descriptor_request {
                // If the clients last descriptor id is the same as the updated descriptor id
                // we'll send the updated status to them.
                if client_descriptor_id == descriptor_id {
                    self.send_message(&message, id);
                }
            }
        });
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;

    // This method is when the lobby is started.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_echo_positions_interval(ctx);
    }
}

impl Lobby {
    fn send_filtered_positions(&self, vhcs: Vec<Vehicle>) {
        self.clients.keys().for_each(|client_id| {
            if let Some(client) = self.clients.get(client_id) {
                if let Some(client_pos) = &client.position {
                    let filtered_vhcs = vhcs
                        .clone()
                        .into_iter()
                        .filter(|vhc| filter_vehicle_position(client_pos, vhc))
                        .collect::<Vec<Vehicle>>();
                    if filtered_vhcs.len() > 0 {
                        self.send_message(
                            &serde_json::to_string(&ServerOutput::VehiclePositions(
                                VehiclePositionsOutput {
                                    timestamp: Lobby::get_current_timestamp(),
                                    vehicles: filtered_vhcs,
                                },
                            ))
                            .unwrap(),
                            client_id,
                        );
                    }
                }
            }
        });
    }
}

impl Handler<EchoPositions> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: EchoPositions, _: &mut Context<Self>) -> Self::Result {
        let vehicle_data = self.trafiklab.get_vehicle_positions().unwrap();

        // Fetch vehicle positions.
        let mut vehicle_positions: Vec<Vehicle> = vehicle_data
            .entity
            .iter()
            .map(|entity| {
                let vehicle = entity.vehicle.as_ref().unwrap();

                let descriptor_id = vehicle
                    .vehicle
                    .as_ref()
                    .unwrap()
                    .id
                    .as_ref()
                    .unwrap()
                    .to_string();

                let trip_id = match vehicle.trip.as_ref() {
                    Some(value) => match value.trip_id.as_ref() {
                        Some(id) => Some(id.to_string()),
                        None => None,
                    },
                    None => None,
                };
                Vehicle {
                    descriptor_id: descriptor_id,
                    trip_id: trip_id,
                    line: None,
                    position: vehicle.position.as_ref().unwrap().clone(),
                }
            })
            .collect();

        let conn = self.db_connection.clone();

        Box::pin(
            async move {
                // Remove all vehicles that are not mapped to a trip_id since they are most likely not in trafic
                vehicle_positions = vehicle_positions
                    .into_iter()
                    .filter(|vehicle| vehicle.trip_id.is_some())
                    .collect();

                for v in vehicle_positions.iter_mut() {
                    if let Some(trip_id) = &v.trip_id {
                        if let Some(trip) = conn.get_trip(doc! {"trip_id": trip_id}).await {
                            if let Some(route) =
                                conn.get_route(doc! {"route_id": trip.route_id}).await
                            {
                                v.line = Some(route.route_short_name);
                            }
                        }
                    }
                }

                vehicle_positions
            }
            .into_actor(self)
            .map(move |positions, act, _ctx| {
                act.send_filtered_positions(positions);
            }),
        )
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "Connect" message.
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        // Store a new clien data object in the clients hashmap.
        self.clients
            .insert(msg.self_id, ClientData::new(msg.self_id, msg.addr));

        println!("Client with id '{}' connected.", msg.self_id);
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "Disconnect" message.
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Try and remove the client from the clients hashmap.
        if self.clients.remove(&msg.self_id).is_some() {
            println!("Client with id '{}' disconnected.", msg.self_id);
        }
    }
}

impl Handler<PositionUpdate> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "PositionUpdate" message.
    fn handle(&mut self, msg: PositionUpdate, _: &mut Context<Self>) {
        let client_data = self.clients.get_mut(&msg.self_id).unwrap();

        println!("Updated position for client with id '{}'", &msg.self_id);

        // Update the client's position to the new position.
        client_data.update_position(msg.position);
    }
}

impl Handler<RouteRequest> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    // This method is called whenever the Lobby receives a "RouteRequest" message.
    fn handle(&mut self, msg: RouteRequest, _: &mut Context<Self>) -> Self::Result {
        println!(
            "Client with id '{}' requested route information for identifier '{}'",
            msg.self_id, &msg.identifier
        );

        // Important to clone this value so it will be accessible inside the async block in the pinned box.
        let client_id = msg.self_id.clone();

        // Note that we also clone a handle to the database connection since "self" cannot be accessed
        // inside the async block. "self" can however be accessed inside the "map" call as "act".
        let conn = self.db_connection.clone();

        Box::pin(
            async move {
                // Check if the line number is not empty and only contains numbers
                if msg.identifier.is_empty() || !only_numbers(&msg.identifier) {
                    return ServerOutput::error_message(
                        ErrorType::RouteInfo,
                        format!("'{}' is not a valid identifier", &msg.identifier),
                    );
                }

                // Determine what query should be used to fetch the shapes with.
                let shape_query = match msg.identifier.len() <= 3 {
                    // If the length is less than or equal to 3, the identifier is a line number.
                    true => {
                        let route_id = match conn
                            .get_route(doc! {"route_short_name": &msg.identifier})
                            .await
                        {
                            Some(route) => route.route_id,
                            None => {
                                return ServerOutput::error_message(
                                    ErrorType::RouteInfo,
                                    format!("'{}' is not a valid line number", &msg.identifier),
                                );
                            }
                        };

                        doc! {"route_id": &route_id}
                    }
                    // Otherwise the identifier is a trip id
                    false => doc! {"trip_id": &msg.identifier},
                };

                // Make a request to the database to figure out what "shape_id" the trip has.
                let shape_id = match conn.get_trip(shape_query).await {
                    Some(trip) => trip.shape_id.to_string(),
                    None => {
                        return ServerOutput::error_message(
                            ErrorType::ServerError,
                            "Unable to retrieve data".to_owned(),
                        );
                    }
                };

                // Make a request to the database to get all "shapes" from the "shape_id".
                let nodes = match conn.get_shapes(doc! {"shape_id": &shape_id}).await {
                    Some(nodes) => nodes,
                    None => {
                        return ServerOutput::error_message(
                            ErrorType::ServerError,
                            "Unable to retrieve data".to_owned(),
                        );
                    }
                };

                // Create the serialized json message that will be sent back to the client.
                serde_json::to_string(&ServerOutput::RouteInformation(RouteInformationOutput {
                    timestamp: Lobby::get_current_timestamp(),
                    route: nodes,
                }))
                .unwrap()
            }
            // Converts future to ActorFuture
            .into_actor(self)
            // message is the value that is returned from the async block above, act is a mutable reference to "self" (the lobby)
            // and ctx is a mutable referenced context with an actor handle to the lobby.
            .map(move |message, act, _ctx| {
                // Send the data back to the client.
                // We don't need to check if the client is still connected here since "send_message" checks this.
                act.send_message(&message, &client_id);
            }),
        )
    }
}

impl Handler<PassengerInfo> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "PassengerInfo" message.
    fn handle(&mut self, msg: PassengerInfo, _: &mut Context<Self>) -> Self::Result {
        println!(
            "Client with id '{}' requested passenger information for '{}'",
            &msg.self_id, &msg.descriptor_id
        );

        // If the descriptor id does not exist in the hashmap we add random data for it.
        if !self.passenger_info.contains_key(&msg.descriptor_id) {
            self.passenger_info.insert(
                msg.descriptor_id.clone(),
                PassengerInformationOutput {
                    passengers: self.rng.gen_range(0..15),
                    capacity: self.rng.gen_range(20..35),
                },
            );
        }

        let passenger_info = self.passenger_info.get(&msg.descriptor_id).unwrap().clone();

        // Send the passenger information to the client.
        self.send_message(
            &serde_json::to_string(&ServerOutput::PassengerInformation(passenger_info)).unwrap(),
            &msg.self_id,
        );

        // Update the clients last descriptor
        self.clients
            .get_mut(&msg.self_id)
            .unwrap()
            .update_last_descriptor(msg.descriptor_id);
    }
}

impl Handler<ReserveSeat> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "ReserveSeat" message.
    fn handle(&mut self, msg: ReserveSeat, _: &mut Context<Self>) -> Self::Result {
        println!(
            "Client with id '{}' reserved a seat on '{}'",
            &msg.self_id, &msg.descriptor_id
        );

        // Keeps track of whether a reservation was made or not.
        let mut reservation_was_made = false;

        match self.passenger_info.get_mut(&msg.descriptor_id) {
            Some(passenger_info) => {
                if passenger_info.passengers + 1 != passenger_info.capacity {
                    reservation_was_made = true;

                    // Increment the passenger count.
                    passenger_info.passengers += 1;

                    let client_data = self.clients.get_mut(&msg.self_id).unwrap();

                    // If the update in the database was successful, store the descriptor id for
                    // the bus on which the seat was reserved so that it can be "unreserved" later.
                    client_data.reserved_seat = Some(msg.descriptor_id.clone());
                } else {
                    self.send_error(
                        &msg.self_id,
                        ErrorType::Reserve,
                        format!(
                            "The bus with descriptor id '{}' is full",
                            &msg.descriptor_id
                        ),
                    );
                }
            }
            None => {
                // If a bus with the descriptor id does not exist in the passenger information hashmap
                // send an error message to the client.
                self.send_error(
                    &msg.self_id,
                    ErrorType::Reserve,
                    format!(
                        "A bus with descriptor id '{}' does not exist.",
                        &msg.descriptor_id
                    ),
                );
            }
        };

        if reservation_was_made {
            // Send updates to all concerned clients.
            self.send_passenger_update(&msg.descriptor_id);
        }
    }
}

impl Handler<UnreserveSeat> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "UnreserveSeat" message.
    fn handle(&mut self, msg: UnreserveSeat, _: &mut Context<Self>) -> Self::Result {
        println!("Client with id '{}' unreserved their seat", &msg.self_id);

        // Keeps track of an unreserved seat, if a seat was unreserved.
        let mut unreserved_seat = String::new();

        let client_data = self.clients.get_mut(&msg.self_id).unwrap();

        match &client_data.reserved_seat {
            Some(descriptor_id) => {
                unreserved_seat = descriptor_id.clone();

                // Decrement the passenger count in the passenger info hashmap.
                self.passenger_info
                    .get_mut(descriptor_id)
                    .unwrap()
                    .passengers -= 1;

                // Remove the reserved seat from the client.
                client_data.reserved_seat = None;
            }
            None => self.send_error(
                &msg.self_id,
                ErrorType::Unreserve,
                "Cannot unreserve since there is no active reservation.".to_owned(),
            ),
        };

        if !unreserved_seat.is_empty() {
            // Send updates to all concerned clients.
            self.send_passenger_update(&unreserved_seat);
        }
    }
}
