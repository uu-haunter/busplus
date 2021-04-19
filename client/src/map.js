import React from "react";
import { useState, useEffect } from "react";
import {
  GoogleMap,
  useLoadScript,
  Marker,
  InfoWindow,
} from "@react-google-maps/api";
import { computeDistanceBetween, interpolate } from 'spherical-geometry-js';
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
  const [realtimeData, setRealtimeData] = useState({timestamp: null, positions: []});
  const [currentCenter, setCurrentCenter] = useState(defaultCenter);
  const [selectedMarker, setSelectedMarker] = useState(null);
  const [markers, setMarkers] = useState([]);

  useEffect(() => {
    const ms = 40; // milliseconds between position updates
    const updateInterval = setInterval(() => {
      // serverUpdateInterval is the time interval at which the client receives realtime updates.
      // If this interval is changed on the server side, serverUpdateInterval has to be changed accordingly.
      // TODO: calculate serverUpdateInterval instead of using a constant value.
      let serverUpdateInterval = 5000;

      // The time that has passed since the last realtime update was received.
      let dt = (new Date().getTime() - realtimeData.timestamp);

      // Dividing the time delta with the time interval of realtime updates
      // in order to get the fraction of the way that the vehicle should have
      // reached if it moves at a constant rate of speed.
      let fraction = dt / serverUpdateInterval;

      if(fraction > 1) return;

      setRealtimeData(
        {
          timestamp: realtimeData.timestamp,
          positions: Object.fromEntries(
            Object.entries(realtimeData.positions).map(([vehicleId, position]) => {
              //let copy = JSON.parse(JSON.stringify(position));

              // interpolate between the source and target positions using the calculated fraction
              // to get the new position.
              let newLatLng = interpolate(position.source, position.target, fraction);

              position.current.latitude = newLatLng.lat();
              position.current.longitude = newLatLng.lng();

              return [vehicleId, position];
            })
          )
        }
      );
    }, ms);

    return () => {
      clearInterval(updateInterval);
    };
  }, [realtimeData, realtimeData.positions]);

  useEffect(() => {
    setRealtimeData(
      {
        timestamp: new Date().getTime(),
        positions: Object.fromEntries(
          props.realtimeData.vehiclePositions.map(bus => {
            let vehicleId = bus.descriptorId.toString();
            let entry = realtimeData.positions[vehicleId];

            return [
              vehicleId,
              {
                source: ( entry ? {...entry.target} : {...bus.position} ),
                current: ( entry ? {...entry.target} : {...bus.position} ),
                target: {...bus.position}
              }
            ];

          })
        )
      }
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
        onClick={()=>{setSelectedMarker(null)}}
        onLoad={onMapLoad}
        onBoundsChanged={onBoundsChanged}
      >
        {
          Object.entries(realtimeData.positions).map(([vehicleId, position]) => (
            <Marker
              key={vehicleId}
              position={{
                lat: position.current.latitude,
                lng: position.current.longitude
              }}
              onClick={() => {setSelectedMarker({id: vehicleId, position: position.current});}}
            >
            </Marker>
          ))

        }
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
      <Fab color="primary" id="themeButton" onClick={changeTheme}>
        <Brightness3Icon />
      </Fab>
    </div>
  );
}

export default Map;
