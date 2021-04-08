import React from 'react';
import GoogleMapReact from 'google-map-react';
import Fab from '@material-ui/core/Fab';
import Brightness3Icon from '@material-ui/icons/Brightness3';
import MyLocationIcon from '@material-ui/icons/MyLocation';
import PersonPinCircleIcon from '@material-ui/icons/PersonPinCircle';
import './App.css';


const defaultLat = 59.8585;
const defaultLng = 17.6389;
const styles = require('./mapstyle.json');
const Marker = () => <div className='marker'>&#128652;</div>;
const CenterMarker = () => <div className='centerMarker'><PersonPinCircleIcon color="secondary"/></div>;

class Map extends React.Component {

	constructor(props) {
		super(props);
		this.state = {
			realtimeData: props.realtimeData, // realtime information about the busses
      theme: styles.day,
      center: {
        lat: defaultLat,
        lng: defaultLng
      }
		};
    this.updateLocation = this.updateLocation.bind(this);
    this.setCoordinates = this.setCoordinates.bind(this);
	};

  changeTheme() {
    if (this.state.theme === styles.day) {
      this.setState({
        realtimeData: this.state.realtimeData, // realtime information about the busses
        theme: styles.night
      });
    } else {
      this.setState({
        realtimeData: this.state.realtimeData, // realtime information about the busses
        theme: styles.day
      });
    }
  };

  updateLocation() {
    if(navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(this.setCoordinates) 
    } else {
      alert("Browser error")
    }
  };

  setCoordinates(position) {
    this.setState({
      center: {
        lat: position.coords.latitude,
        lng: position.coords.longitude
      }
    })
  };


	static defaultProps = {
		// Coordinates of Stora torget, Uppsala
    center: {
      lat: defaultLat,
      lng: defaultLng
    },
    zoom: 14
  };

	componentDidMount() {};



	render() {
    
		return (
			<div style={{ height: '100vh', width: '100%' }}>
        <GoogleMapReact
          bootstrapURLKeys={{ key: '' }}
          defaultCenter={this.props.center}
          defaultZoom={this.props.zoom}
          center={this.state.center}
					options={{
					 styles: this.state.theme,
					 disableDefaultUI: true,
					 fullscreenControl: false
				 	}}
        >
          {
            this.state.realtimeData.map((obj, i) => (
             <Marker
                key={obj.id}
                lat={obj.vehicle.position.latitude}
                lng={obj.vehicle.position.longitude}

              />
            ))
          }

          <CenterMarker
            lat = {this.state.center.lat}
            lng = {this.state.center.lng}
          />
          
        </GoogleMapReact>
        <Fab
	        	id = "locationButton"
	        	color = "primary"
	        	aria-label = "locationButton"
            onClick = {this.updateLocation}>
	          <MyLocationIcon/>
        </Fab>
        <Fab 
          color = "primary" 
          id = "themeButton" 
          onClick = {() => this.changeTheme()}>
          <Brightness3Icon/>
        </Fab>
      </div>
		);
	};
}

export default Map;
