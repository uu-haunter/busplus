import React from "react";
import { useState, useEffect } from "react";
import {
  GoogleMap,
  useLoadScript,
  Marker,
  InfoWindow,
  Polyline,
} from "@react-google-maps/api";
import { computeDistanceBetween, interpolate } from "spherical-geometry-js";
import Fab from "@material-ui/core/Fab";
import Button from "@material-ui/core/Button";
import Brightness3Icon from "@material-ui/icons/Brightness3";
import MyLocationIcon from "@material-ui/icons/MyLocation";
import "./App.css";

// Returns a route request message for a specific bus line
export function routeRequest(lineNo) {
  return {
    type: "get-route-info",
    payload: {
      line: lineNo,
    },
  };
}
// Message to reserve seat
function reserveSeatRequest(vehicleId) {
  return {
    type: "reserve-seat",
    payload: {
      id: vehicleId,
    },
  };
}

// Message to unreserve seat
function unreserveSeatRequest() {
  return {
    type: "unreserve-seat",
  };
}

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

  // State-variables
  const styles = require("./mapstyle.json");
  const [activeReservation, setReservation] = useState(false);
  const [currentTheme, setCurrentTheme] = useState(styles.day);
  const [vehicleData, setVehicleData] = useState({
    timestamp: null,
    vehicles: {},
  });
  const [currentCenter, setCurrentCenter] = useState(defaultCenter);
  const [selectedMarker, setSelectedMarker] = useState(null);
  const [currentRoute, setRoute] = useState(null);

  // Options that specify the route-drawing
  const polyLineOptions = {
    strokeColor: "#FF0000",
    strokeOpacity: 0.8,
    strokeWeight: 2,
    fillColor: "#FF0000",
    fillOpacity: 0.35,
    clickable: false,
    draggable: false,
    editable: false,
    visible: true,
    radius: 30000,
    zIndex: 1,
  };

  // Hook used to animate buses smoother
  useEffect(() => {
    const ms = 40; // milliseconds between position updates
    const updateInterval = setInterval(() => {
      // serverUpdateInterval is the time interval at which the client receives realtime updates.
      // If this interval is changed on the server side, serverUpdateInterval has to be changed accordingly.
      // TODO: calculate serverUpdateInterval instead of using a constant value.
      let serverUpdateInterval = 5000;

      // The time that has passed since the last realtime update was received.
      let dt = new Date().getTime() - vehicleData.timestamp;

      // Dividing the time delta with the time interval of realtime updates
      // in order to get the fraction of the way that the vehicle should have
      // reached if it moves at a constant rate of speed.
      let fraction = dt / serverUpdateInterval;

      if (fraction > 1) return;

      setVehicleData({
        timestamp: vehicleData.timestamp,
        vehicles: Object.fromEntries(
          Object.entries(vehicleData.vehicles).map(([vehicleId, vehicle]) => {
            // interpolate between the source and target positions using the calculated fraction
            // to get the new position.
            let newLatLng = interpolate(
              vehicle.sourcePosition,
              vehicle.targetPosition,
              fraction
            );

            vehicle.currentPosition.latitude = newLatLng.lat();
            vehicle.currentPosition.longitude = newLatLng.lng();

            return [vehicleId, vehicle];
          })
        ),
      });
    }, ms);

    return () => {
      clearInterval(updateInterval);
    };
  }, [vehicleData, vehicleData.vehicles]);

  useEffect(() => {
    setVehicleData({
      timestamp: new Date().getTime(),
      vehicles: Object.fromEntries(
        props.realtimeData.map((vehicle) => {
          let vehicleId = vehicle.descriptorId.toString();
          let entry = vehicleData.vehicles[vehicleId];

          return [
            vehicleId,
            {
              line: vehicle.line,
              sourcePosition: entry
                ? { ...entry.targetPosition }
                : { ...vehicle.position },
              currentPosition: entry
                ? { ...entry.targetPosition }
                : { ...vehicle.position },
              targetPosition: { ...vehicle.position },
            },
          ];
        })
      ),
    });
  }, [props.realtimeData]);

  // Hook used to modify the route-data
  useEffect(() => {
    setRoute(props.route);
  }, [props.route]);

  // Update the position every second
  useEffect(() => {
    const interval = setInterval(() => {
      onBoundsChanged();
      //TODO: Maybe update userposition here
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const mapRef = React.useRef();
  const onMapLoad = React.useCallback((map) => {
    mapRef.current = map;
  }, []);

  // called when the maps bounds are changed e.g. when a user drags the map
  const onBoundsChanged = () => {
    let lat = mapRef.current.getCenter().lat();
    let lng = mapRef.current.getCenter().lng();
    let radius = getBoundingSphereRadius();

    let message = {
      type: "geo-position-update",
      payload: {
        maxDistance: radius,
        position: {
          type: "Point",
          coordinates: [lat, lng],
        },
      },
    };

    props.wsSend(JSON.stringify(message));
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
        zoom={16}
        center={currentCenter}
        mapContainerStyle={mapContainerStyle}
        options={options}
        onClick={() => {
          setSelectedMarker(null);
          setRoute([]);
        }}
        onLoad={onMapLoad}
        onBoundsChanged={onBoundsChanged}
      >
        {Object.entries(vehicleData.vehicles).map(([vehicleId, vehicle]) => (
          <Marker
            key={vehicleId}
            position={{
              lat: vehicle.currentPosition.latitude,
              lng: vehicle.currentPosition.longitude,
            }}
            onClick={() => {
              setSelectedMarker(vehicleId);
              // TODO: Change argument for routeRequest when we have line data
              props.wsSend(JSON.stringify(routeRequest(vehicle.line)));
            }}
            icon={{
              url: "/bus.svg",
              origin: new window.google.maps.Point(0, 0),
              anchor: new window.google.maps.Point(15, 15),
              scaledSize: new window.google.maps.Size(30, 30),
            }}
          ></Marker>
        ))}
        {selectedMarker && (
          <InfoWindow
            position={{
              lat:
                vehicleData.vehicles[selectedMarker].currentPosition.latitude,
              lng:
                vehicleData.vehicles[selectedMarker].currentPosition.longitude,
            }}
            onCloseClick={() => {
              setSelectedMarker(null);
            }}
          >
            <div>
              <p>{`Bus ${vehicleData.vehicles[selectedMarker].line} \n Passengers ${vehicleData.vehicles[selectedMarker].passengers} / ${vehicleData.vehicles[selectedMarker].capacity}`}</p>
              {!activeReservation ? (
                <Button
                  variant="outlined"
                  color="primary"
                  onClick={() => {
                    setReservation(true);
                    props.wsSend(
                      JSON.stringify(reserveSeatRequest(selectedMarker))
                    );
                  }}
                >
                  Reserve Seat
                </Button>
              ) : (
                <Button
                  variant="contained"
                  color="secondary"
                  onClick={() => {
                    setReservation(false);
                    props.wsSend(
                      JSON.stringify(unreserveSeatRequest())
                    );
                  }}
                >
                  cancel reservation
                </Button>
              )}
            </div>
          </InfoWindow>
        )}

        <Marker
          position={{
            lat: currentCenter.lat,
            lng: currentCenter.lng,
          }}
          onClick={() => {
            mapRef.current.setZoom(18);
            updateLocation();
          }}
          icon={{
            url: "/circle.svg",
            origin: new window.google.maps.Point(0, 0),
            anchor: new window.google.maps.Point(15, 15),
            //Change Size to (150,150) when using pulsating circle icon
            scaledSize: new window.google.maps.Size(150, 150),
          }}
        />

        {currentRoute && (
          <Polyline
            path={currentRoute.map((obj) => {
              return {
                // TODO: Message should send coords in number format instead of string
                lat: parseFloat(obj.lat),
                lng: parseFloat(obj.lng),
              };
            })}
            options={polyLineOptions}
          />
        )}
      </GoogleMap>
      <Fab
        id="locationButton"
        color="primary"
        aria-label="locationButton"
        onClick={() => {
          mapRef.current.setZoom(18);
          updateLocation();
        }}
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
