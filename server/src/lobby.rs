//! Keeps track of all connected clients and a shared state.

use std::collections::HashMap;
use std::time::Duration;

use actix::prelude::{
    Actor, ActorFuture, Context, Handler, Recipient, ResponseActFuture, WrapFuture,
};
use actix::AsyncContext;
use mongodb::bson::doc;
use uuid::Uuid;

use crate::client::ClientData;
use crate::config::{Config, CONFIG_FILE_PATH};
use crate::database::DbConnection;
use crate::gtfs::trafiklab::TrafiklabApi;
use crate::messages::{
    Connect, Disconnect, PositionUpdate, ReserveSeat, RouteRequest, UnreserveSeat, WsMessage,
};
use crate::protocol::server_protocol::{
    ErrorOutput, ErrorType, RouteInformationOutput, ServerOutput, Vehicle, VehiclePositionsOutput,
};

use crate::util::filter_vehicle_position;

/// The interval in which data is fetched from the external Trafiklab API and
/// echoed out to all connected users.
const API_FETCH_INTERVAL: Duration = Duration::from_secs(5);

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
        ctx.run_interval(self.echo_positions_interval, |act, _| {
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

            let vehicle_data = act.trafiklab.get_vehicle_positions().unwrap();

            // TODO: Instead of collecting all data in a big chunk like this,
            // the data should be tailored depending on what buses the user can see
            // in regards to their "position".

            let mut vehicle_positions = vehicle_data
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
                        position: vehicle.position.as_ref().unwrap().clone(),
                    }
                })
                .collect();
            act.send_filtered_positions(vehicle_positions);
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

impl Handler<Connect> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "Connect" message.
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        // Store a new clien data object in the clients hashmap.
        self.clients
            .insert(msg.self_id, ClientData::new(msg.self_id, msg.addr));

        // TODO: Remove this println. Only here to show that events occur.
        println!("Client with id '{}' connected.", msg.self_id);
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    // This method is called whenever the Lobby receives a "Disconnect" message.
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Try and remove the client from the clients hashmap.
        if self.clients.remove(&msg.self_id).is_some() {
            // TODO: Remove this println. Only here to show that events occur.
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
            "Client with id '{}' requested line information for line '{}'",
            msg.self_id, &msg.line_number
        );

        // Important to clone these values so they will be accessible inside the async block in the
        // pinned box.
        let line_number = msg.line_number.clone();
        let client_id = msg.self_id.clone();

        // Note that we also clone a handle to the database connection since "self" cannot be accessed
        // inside the async block. "self" can however be accessed inside the "map" call as "act".
        let conn = self.db_connection.clone();

        Box::pin(
            async move {
                // Check if the line number is not empty and only contains numbers
                if msg.line_number.is_empty() || !msg.line_number.chars().all(|c| c.is_numeric()) {
                    return ServerOutput::error_message(
                        ErrorType::RouteInfo,
                        format!("'{}' is not a valid line number", line_number),
                    );
                }

                let route_id = match conn
                    .get_route(doc! {"route_short_name": &line_number})
                    .await
                {
                    Some(route) => route.route_id,
                    None => {
                        return ServerOutput::error_message(
                            ErrorType::RouteInfo,
                            format!("'{}' is not a valid line number", line_number),
                        );
                    }
                };

                // Make a request to the database to figure out what "shape_id" the route has.
                let shape_id = match conn.get_trip(doc! {"route_id": &route_id}).await {
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
                    line: msg.line_number,
                    route_id: route_id,
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

impl Handler<ReserveSeat> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    // This method is called whenever the Lobby receives a "ReserveSeat" message.
    fn handle(&mut self, msg: ReserveSeat, _: &mut Context<Self>) -> Self::Result {
        println!(
            "Client with id '{}' reserved a seat on '{}'",
            &msg.self_id, &msg.descriptor_id
        );

        Box::pin(
            async move {
                // Update data about bus with 'descriptor_id' in the database
                // (increment expected_passenger_count by 1).
            }
            .into_actor(self)
            .map(move |_, act, _| {
                let client_data = act.clients.get_mut(&msg.self_id).unwrap();

                // If the update in the database was successful, store the descriptor id for
                // the bus on which the seat was reserved so that it can be "unreserved" later.
                client_data.reserved_seat = Some(msg.descriptor_id);
            }),
        )
    }
}

impl Handler<UnreserveSeat> for Lobby {
    type Result = ResponseActFuture<Self, ()>;

    // This method is called whenever the Lobby receives a "UnreserveSeat" message.
    fn handle(&mut self, msg: UnreserveSeat, _: &mut Context<Self>) -> Self::Result {
        println!("Client with id '{}' unreserved their seat", &msg.self_id);

        let reserved_seat = self
            .clients
            .get_mut(&msg.self_id)
            .unwrap()
            .reserved_seat
            .clone();

        Box::pin(
            async move {
                if reserved_seat.is_none() {
                    return None;
                }

                // Update data about bus with 'descriptor_id' in the database
                // (decrement expected_passenger_count by 1).

                reserved_seat
            }
            .into_actor(self)
            .map(move |result, act, _| {
                // Only if the returned result is Some, should the clients reserved seat
                // be set back to None.
                if result.is_some() {
                    let client_data = act.clients.get_mut(&msg.self_id).unwrap();

                    // Remove the previously stored reserved seat.
                    client_data.reserved_seat = None;
                }
            }),
        )
    }
}
