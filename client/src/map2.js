import React from "react";
import { useState } from "react";
import GoogleMapReact from "google-map-react";
import Fab from "@material-ui/core/Fab";
import Brightness3Icon from "@material-ui/icons/Brightness3";
import MyLocationIcon from "@material-ui/icons/MyLocation";
import PersonPinCircleIcon from "@material-ui/icons/PersonPinCircle";
import "./App.css";

function Map2(props) {
  const defaultLat = 59.8585;
  const defaultLng = 17.6389;
  const defaultCenter = {
    lat: defaultLat,
    lng: defaultLng,
  };
  const styles = require("./mapstyle.json");
  const Marker = () => <div className="marker">&#128652;</div>;
  const CenterMarker = () => (
    <div className="centerMarker">
      <PersonPinCircleIcon color="secondary" />
    </div>
  );

  const [currentTheme, setCurrentTheme] = useState(styles.day);
  const [realtimeData, setRealtimeData] = useState(props.realtimeData);
  const [currentCenter, setCurrentCenter] = useState(defaultCenter);

  const updateLocation = () => {
    if(navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(setCoordinates);
    } else {
      alert("Browser error");
    }
  };

  const setCoordinates = (position) => {
    setCurrentCenter({
      lat: position.coords.latitude,
      lng: position.coords.longitude
    });
  };

  const changeTheme = () => {
    if (currentTheme === styles.day) {
        setCurrentTheme(styles.night)
    } else {
        setCurrentTheme(styles.day)
    }
  }

  return (
    <div style={{ height: "100vh", width: "100%" }}>
      <GoogleMapReact
        bootstrapURLKeys={{ key: "AIzaSyD2nxWN__H5vzHRFdYejT9v8_qLv9R8vUk" }}
        defaultCenter={defaultCenter}
        defaultZoom={15}
        center={currentCenter}
        options={{
          styles: currentTheme,
          disableDefaultUI: true,
          fullscreenControl: false,
        }}
      >
        {realtimeData.map((obj, i) => (
          <Marker
            key={obj.id}
            lat={obj.vehicle.position.latitude}
            lng={obj.vehicle.position.longitude}
          />
        ))}

        <CenterMarker 
        lat={currentCenter.lat} 
        lng={currentCenter.lng} />

      </GoogleMapReact>
      <Fab
        id="locationButton"
        color="primary"
        aria-label="locationButton"
        onClick={updateLocation}
      >
        <MyLocationIcon />
      </Fab>
      <Fab
        color="primary"
        id="themeButton"
        onClick = {changeTheme}
      >
        <Brightness3Icon />
      </Fab>
    </div>
  );
}

export default Map2;
