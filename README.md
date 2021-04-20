# Bus+

## Quick Start
In order for the server to work you need to create a config file in the root directory of the repository once you clone it. The config file should be named `config.yml` and should *at least* contain the following keys:

```yml
trafiklab_api:
  realtime_key: <very secret api key>
  static_key: <very secret api key>

database:
  uri: <very secret connection uri>
```

## Documentation

In order to generate documentation for the server code that is written in Rust, navigate to the `server` directory and run:

```bash
# Generates documentation in "target/doc/server/"
cargo doc --no-deps

# Add "--open" to open the documentation in a browser when it's been generated.
cargo doc --no-deps --open
```
