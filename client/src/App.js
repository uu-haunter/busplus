import React from "react";
import Map from "./map.js";
import LoadingScreen from "./LoadingScreen.js";
import "./App.css";
import { geoPositionUpdateRequest } from "./messages.js";

class App extends React.Component {
  constructor() {
    super();
    this.state = {
      realtimeData: [],
      route: [],
      passengerData: { passengers: 0, capacity: 0 },
      vehiclesLoaded: false,
    };
    this.ws = null;
    this.wsSend = this.wsSend.bind(this);
  }

  wsSend(message) {
    if (this.ws) {
      console.log("Sending message", message);
      this.ws.send(message);
    }
  }

  handleReceivedMessage(message) {
    if (message.type === "vehicle-positions") {
      this.setState({ vehiclesLoaded: true });
      this.setState({ realtimeData: message.payload.vehicles });
    } else if (message.type === "route-info") {
      console.log(message);
      this.setState({ route: message.payload.route });
    } else if (message.type === "passenger-info") {
      this.setState({
        passengerData: {
          passengers: message.payload.passengers,
          capacity: message.payload.capacity,
        },
      });
    }

    // Handle other types of messages here
  }

  componentDidMount() {
    let ws = new WebSocket("ws://localhost:8080/ws");
    ws.onopen = () => {
      this.ws = ws;
      console.log("Connected!");

      this.wsSend(
        JSON.stringify(geoPositionUpdateRequest(1000, 59.8585, 17.6389))
      );
    };
    ws.onmessage = (event) => {
      console.log("Message received", event.data);
      try {
        let message = JSON.parse(event.data);
        this.handleReceivedMessage(message);
      } catch (e) {
        console.log("Received message is not JSON");
      }
    };
    ws.onerror = () => {
      console.log("Connection error");
    };
    ws.onclose = () => {
      console.log("Connection closed");
    };
  }

  render() {
    return (
      <div className="App">
        {!this.state.vehiclesLoaded ? (
          <LoadingScreen />
        ) : (
          <div className="App-header">
            <Map
              wsSend={this.wsSend}
              realtimeData={this.state.realtimeData}
              route={this.state.route}
              passengerData={this.state.passengerData}
            />
          </div>
        )}
      </div>
    );
  }
}

export default App;
