# Bus+

Bus+ is a service that lets users make smart and planned choices when traveling. Instead of planning a trip only from table times, Bus+ allows you to see the current location of all buses in you district as well as having support for getting and indication of how many people are traveling on a given bus at any given moment.

The goal of Bus+ is to create a standalone application that can either work as a complement to existing public-transport applications or be merged together with one of them  to create a better experience for travelers.

---

<center>

Light Theme                |  Dark Theme
:-------------------------:|:-------------------------:
![](https://user-images.githubusercontent.com/21147276/117493528-62e1d800-af73-11eb-849f-bc1f593b0c29.png)  |  ![](https://user-images.githubusercontent.com/21147276/117493551-6bd2a980-af73-11eb-89f9-8e9bc2f3ee78.png)

</center>

## Quick Start
In order for the server to work you need to create a config file in the root directory of the repository once you clone it. The config file should be named `config.yml` and should *at least* contain the following keys:

```yml
trafiklab_api:
  realtime_key: <very secret api key>
  static_key: <very secret api key>

  # How often vehicle positions should be fetched from the external API and sent
  # to all connected clients.
  echo_interval: <interval in seconds as a f64 (like 2.0 or 0.667 for example)>

database:
  uri: <very secret connection uri>
```

### Google Maps API

The client application needs a Google Maps API key in order to render the map on each client. Before building the client code, place a file called `.env` in the `client/` directory containing the following line:
```
REACT_APP_GOOGLE_MAPS_API_KEY=<key>
```

### Database

This project relies on static data about routes, shapes and trips from [Trafiklab's Static API](https://www.trafiklab.se/api/gtfs-regional-static-data-beta). As of the latest version of this project, this data must be fetched manually and inserted into a MongoDB database, in a database called `trafiklab-static-api` with the collections `routes`, `shapes` and `trips`. The most efficient way to insert this data is to either use MongoDB's command line tool to import CSV files, or insert them using the GUI [MongoDb Compass](https://www.mongodb.com/products/compass).

If the process of importing data is to be programmatically implemented, one needs to be aware of what types the values have in MongoDB since both Rust and MongoDB is very picky about what types can be used both implicitly and explicitly.

## Building and Running

Commands are executed in the `server/` and `client/` folder respectively.

### Server

In order to build and/or run the server, run:

```bash
# Builds and runs the server application
cargo run

# Only builds the application
cargo build
```

### Client

Run the following command to run the client in development mode:
```bash
npm run start
```

Run the following command in order to build the client code and output it into the `client/build/` directory:
```bash
npm run build
```


## Documentation

### Server

In order to generate documentation for the server code that is written in Rust, navigate to the `server` directory and run:

```bash
# Generates documentation in "target/doc/server/"
cargo doc --no-deps

# Add "--open" to open the documentation in a browser when it's been generated.
cargo doc --no-deps --open
```

### Communication protocol

In `protocol-documentation.md` there is a full description of what messages are sent between the client and the server.
