import React from "react";
import { useState, useEffect } from "react";
import {
  GoogleMap,
  useLoadScript,
  Marker,
  InfoWindow,
} from "@react-google-maps/api";
import { computeDistanceBetween } from "spherical-geometry-js";
import Fab from "@material-ui/core/Fab";
import Brightness3Icon from "@material-ui/icons/Brightness3";
import MyLocationIcon from "@material-ui/icons/MyLocation";
import "./App.css";

/*
 * Function component for the Map of the application
 */

function Map(props) {
  const defaultLat = 59.8585;
  const defaultLng = 17.6389;
  const defaultCenter = {
    lat: defaultLat,
    lng: defaultLng,
  };
  const styles = require("./mapstyle.json");

  // State-variables
  const [currentTheme, setCurrentTheme] = useState(styles.day);
  const [realtimeData, setRealtimeData] = useState(props.realtimeData);
  const [currentCenter, setCurrentCenter] = useState(defaultCenter);
  const [selectedMarker, setSelectedMarker] = useState(null);
  const [markers, setMarkers] = useState([]);

  // Fires a re-render to re-draw each bus on every 
  // API-reponse recieved from the server
  useEffect(() => {
    setRealtimeData(props.realtimeData);
    setMarkers(
      props.realtimeData.map((bus) => (
        <Marker
          key={bus.id}
          position={{
            lat: bus.position.latitude,
            lng: bus.position.longitude,
          }}
          onClick={() => {
            setSelectedMarker(bus);
          }}
          icon={{
            url: "/bus.svg",
            origin: new window.google.maps.Point(0, 0),
            anchor: new window.google.maps.Point(15, 15),
            scaledSize: new window.google.maps.Size(30, 30),
          }}
        ></Marker>
      ))
    );
  }, [props.realtimeData]);

  const mapRef = React.useRef();
  const onMapLoad = React.useCallback((map) => {
    mapRef.current = map;
  }, []);

  // called when the maps bounds are changed e.g. when a user drags the map
  const onBoundsChanged = () => {
    // TODO: uncomment this code once the server supports 'geo-position-update'
    /*
    let lat = mapRef.current.getCenter().lat();
    let lng = mapRef.current.getCenter().lng();
    let radius = getBoundingSphereRadius();

    let message = {
      "type": "geo-position-update",
      "payload": {
        "maxDistance": radius,
        "position": {
          "type": "Point",
          "coordinates": [lat, lng]
        }
      }
    };

    props.wsSend(JSON.stringify(message));
    */
  };

  // returns the radius of the maps bounding sphere in meters
  const getBoundingSphereRadius = () => {
    let center = mapRef.current.getBounds().getCenter();
    let northEast = mapRef.current.getBounds().getNorthEast();

    // return the distance along the earths surface
    return computeDistanceBetween(center, northEast);
  };

  const { isLoaded, loadError } = useLoadScript({
    // Reads the google-maps api_key from your locally created .env file
    googleMapsApiKey: process.env.REACT_APP_GOOGLE_MAPS_API_KEY,
  });

  // Container size for the GoogleMap component
  const mapContainerStyle = {
    height: "100vh",
    width: "100vw",
  };

  // Default options of the GoogleMap component
  const options = {
    styles: currentTheme,
    disableDefaultUI: true,
  };

  // Gets the users position using the browser location
  const updateLocation = () => {
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(setCoordinates);
    } else {
      alert("Browser error");
    }
  };

  // Sets the center of the map to the user-position
  const setCoordinates = (position) => {
    setCurrentCenter({
      lat: position.coords.latitude,
      lng: position.coords.longitude,
    });
  };

  // Changes between dark-theme and light-theme
  const changeTheme = () => {
    if (currentTheme === styles.day) {
      setCurrentTheme(styles.night);
    } else {
      setCurrentTheme(styles.day);
    }
  };

  if (loadError) return "Error";
  if (!isLoaded) return "Loading...";

  return (
    <div>
      <GoogleMap
        zoom={15}
        center={currentCenter}
        mapContainerStyle={mapContainerStyle}
        options={options}
        onClick={() => {
          setSelectedMarker(null);
        }}
        onLoad={onMapLoad}
        onBoundsChanged={onBoundsChanged}
      >
        {markers}
        {selectedMarker && (
          <InfoWindow
            position={{
              lat: selectedMarker.position.latitude,
              lng: selectedMarker.position.longitude,
            }}
            onCloseClick={() => {
              setSelectedMarker(null);
            }}
          >
            <div>
              <p>{`Bus ${selectedMarker.id} \n Passengers ${selectedMarker.passengers} / ${selectedMarker.capacity}`}</p>
            </div>
          </InfoWindow>
        )}

        <Marker
          position={{
            lat: currentCenter.lat,
            lng: currentCenter.lng,
          }}
          icon={{
            url: "/personpin.svg",
            origin: new window.google.maps.Point(0, 0),
            anchor: new window.google.maps.Point(15, 15),
            scaledSize: new window.google.maps.Size(30, 30),
          }}
        />
      </GoogleMap>
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
        onClick={changeTheme}>
        <Brightness3Icon />
      </Fab>
    </div>
  );
}

export default Map;
