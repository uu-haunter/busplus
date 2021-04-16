import React from "react";
import { useState, useEffect } from "react";
import {
  GoogleMap,
  useLoadScript,
  Marker,
  InfoWindow,
} from "@react-google-maps/api";
import { computeDistanceBetween } from 'spherical-geometry-js';
import { Polyline } from '@react-google-maps/api';
import Fab from "@material-ui/core/Fab";
import Brightness3Icon from "@material-ui/icons/Brightness3";
import MyLocationIcon from "@material-ui/icons/MyLocation";
import "./App.css";


function Map(props) {
  const defaultLat = 59.8585;
  const defaultLng = 17.6389;
  const defaultCenter = {
    lat: defaultLat,
    lng: defaultLng,
  };
  const styles = require("./mapstyle.json");
  const [currentTheme, setCurrentTheme] = useState(styles.day);
  const [realtimeData, setRealtimeData] = useState(props.realtimeData);
  const [currentCenter, setCurrentCenter] = useState(defaultCenter);
  const [selectedMarker, setSelectedMarker] = useState(null);
  const [markers, setMarkers] = useState([]);
  const [currentRoute, setRoute] = useState([]);

  const polyLineOptions = {
    strokeColor: '#FF0000',
    strokeOpacity: 0.8,
    strokeWeight: 2,
    fillColor: '#FF0000',
    fillOpacity: 0.35,
    clickable: false,
    draggable: false,
    editable: false,
    visible: true,
    radius: 30000,
    paths: [
      {lat: 37.772, lng: -122.214},
      {lat: 21.291, lng: -157.821},
      {lat: -18.142, lng: 178.431},
      {lat: -27.467, lng: 153.027}
    ],
    zIndex: 1
  };

  const routeRequest = (lineNo) => {
    return {
        "type": "get-route-info",
        "payload": {
          "line": lineNo
        }
    };
  };

  useEffect(() => {
    //setRealtimeData(props.realtimeData);
    setMarkers(
      props.realtimeData.map(obj => (
        <Marker
          key={obj.id}
          position={{
            lat: obj.position.latitude,
            lng: obj.position.longitude,
          }}
          onClick={() => {
            setSelectedMarker(obj);
            props.wsSend(JSON.stringify(routeRequest(obj.id))); //TODO: replace obj.id with line number
            /*setRoute([
              {lat: 59.8585, lng: 17.6389},
              {lat: 59.9585, lng: 17.6389}
            ]);*/
          }}
        >
        </Marker>
      ))
    );
  }, [props.realtimeData]);

  useEffect(() => {
    setRoute(props.route);
  }, [props.route]);

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
  }

  const { isLoaded, loadError } = useLoadScript({
    googleMapsApiKey: "",
  });

  const mapContainerStyle = {
    height: "100vh",
    width: "100vw",
  };

  const options = {
    styles: currentTheme,
    disableDefaultUI: true,
  };

  const updateLocation = () => {
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(setCoordinates);
    } else {
      alert("Browser error");
    }
  };

  const setCoordinates = (position) => {
    setCurrentCenter({
      lat: position.coords.latitude,
      lng: position.coords.longitude,
    });
  };

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
        onClick={()=>{
          setSelectedMarker(null);
          setRoute([]);
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

        <Polyline
        path={currentRoute}
        options={polyLineOptions}
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
      <Fab color="primary" id="themeButton" onClick={changeTheme}>
        <Brightness3Icon />
      </Fab>
    </div>
  );
}

export default Map;
