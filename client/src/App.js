import React from 'react';
import Map from "./map.js";
import Map2 from "./map2.js";
import SearchBar from "./SearchBar.js";
import './App.css';

class App extends React.Component {

	constructor() {
		super();
		this.state = {
			realtimeData: []
		};
	};

	componentDidMount() {
		let ws = new WebSocket("ws://localhost:8080/ws");
		ws.onopen = () => {
			console.log('Connected!');
			ws.send('Echo');
		};
		ws.onmessage = event => {
			console.log('Message received');
			console.log(event.data);

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
	      	<Map2
						realtimeData={this.state.realtimeData}
					/>
	        <SearchBar/>
	      </div>
	    </div>
		);
	};

}

export default App;

