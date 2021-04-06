import React from 'react';
import GoogleMapReact from 'google-map-react';


const BusMarker = ({ text }) => <div className="bus">&#128652;{text}</div>;

const styles = require('./mapstyle.json')

class Map extends React.Component {
  constructor() {
    super();
    this.state = {
      busses: []
    };
  };

  static defaultProps = {
    center: {
      lat: 59.8585,
      lng: 17.6389
    },
    zoom: 14
  };

  componentDidMount() {

  }

  render() {
    return (
      <div style={{ height: '100vh', width: '100%' }}>
        <GoogleMapReact
          bootstrapURLKeys={{ key: 'AIzaSyCcyI7RMA8UmIBweEn6tKpbjY-sI01Sbss' }}
          defaultCenter={this.props.center}
          defaultZoom={this.props.zoom}
          options={{
            styles: styles,
            disableDefaultUI: true,
            fullscreenControl: false
          }}
        >
          {
            this.state.busses.map((bus, i) => (
             <BusMarker
                key={bus.id}
                lat={bus.vehiclePosition.latitude}
                lng={bus.vehiclePosition.longitude}
                text={bus.shortRouteName}
              />
            ))
          }
        </GoogleMapReact>
      </div>
    );
  }
}

export default Map;