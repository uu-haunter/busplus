import React from 'react';
import Map from "./map.js";
import SearchBar from "./SearchBar.js";
import './App.css';

class App extends React.Component {

	constructor() {
		super();
		this.state = {
			realtimeData: []
		};
		this.ws = null;
		this.wsSend = this.wsSend.bind(this);
	};

	wsSend(message) {
		if(this.ws) {
			console.log("Sending message", message);
			this.ws.send(message);
		}
	};

	componentDidMount() {
		let ws = new WebSocket("ws://localhost:8080/ws");
		ws.onopen = () => {
			this.ws = ws;
			console.log('Connected!');
		};
		ws.onmessage = event => {
			console.log('Message received', event.data);
			// handle received data here
		};
		ws.onerror = () => {
			console.log('Connection error');
		};
		ws.onclose = () => {
			console.log('Connection closed');
		};
	};

	render() {
		return (
			<div className="App">
	      <div className = "App-header">
	      	<Map
						wsSend={this.wsSend}
						realtimeData={this.state.realtimeData}
					/>
	        <SearchBar/>
	      </div>
	    </div>
		);
	};

}

export default App;
