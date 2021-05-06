import React from "react";
import { useState, useEffect, useReducer } from "react";
import {
  GoogleMap,
  useLoadScript,
  Marker,
  MarkerClusterer,
  InfoWindow,
  Polyline,
} from "@react-google-maps/api";
import { computeDistanceBetween, interpolate } from "spherical-geometry-js";
import {
  routeRequest,
  geoPositionUpdateRequest,
  reserveSeatRequest,
  unreserveSeatRequest,
  getPassengerInfo,
} from "./messages.js";
import SearchBar from "./SearchBar.js";
import Fab from "@material-ui/core/Fab";
import Button from "@material-ui/core/Button";
import Brightness3Icon from "@material-ui/icons/Brightness3";
import MyLocationIcon from "@material-ui/icons/MyLocation";
import "./App.css";

// the maps default latitude, longitude and center
const defaultLat = 59.8585;
const defaultLng = 17.6389;
const defaultCenter = {
  lat: defaultLat,
  lng: defaultLng,
};

// the maps styling
const styles = require("./mapstyle.json");

// Styling for the maps container
const mapContainerStyle = {
  height: "100vh",
  width: "100vw",
};

// Styling for the polyline that is shown when drawing
// a route
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

// options for the vehicle marker cluster
const markerClusterOptions = {
  imagePath: null,
  ignoreHidden: true,
  imageSizes: [0, 0, 0, 0, 0],
};

function updateNavigatorGeolocation(callback) {
  if (navigator.geolocation) {
    navigator.geolocation.getCurrentPosition(callback);
  } else {
    alert("Browser error");
  }
}

// linear interpolation between two angles
function lerpDegrees(source, target, amount) {
  let angle = target - source;
  if (angle > 180) angle -= 360;
  else if (angle < -180) angle += 360;
  return source + angle * amount;
}

function vehicleDataReducer(state, action) {
  if (action.type === "setNewData") {
    let now = new Date().getTime();
    let serverUpdateInterval = now - state.timestamp;

    let vehicles = (state.selectedVehicle.line
      ? action.payload.filter(
          (vehicle) => vehicle.line === state.selectedVehicle.line
        )
      : action.payload
    ).map((vehicle) => {
      let vehicleId = vehicle.descriptorId.toString();
      let entry = state.vehicles[vehicleId];

      return [
        vehicleId,
        {
          trip: vehicle.tripId,
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
    });

    return {
      ...state,
      timestamp: now,
      serverUpdateInterval: serverUpdateInterval,
      vehicles: Object.fromEntries(vehicles),
    };
  }

  if (action.type === "animate") {
    // The time that has passed since the last realtime update was received.
    let dt = new Date().getTime() - state.timestamp;

    // Dividing the time delta with the time interval of realtime updates
    // in order to get the fraction of the way that the vehicle should have
    // reached if it moves at a constant rate of speed.
    let fraction = (dt * 1.0) / state.serverUpdateInterval;

    if (fraction > 1) return state;

    return {
      ...state,
      vehicles: Object.fromEntries(
        Object.entries(state.vehicles).map(([vehicleId, vehicle]) => {
          // interpolate between the source and target positions using the calculated fraction
          // to get the new position.
          let newLatLng = interpolate(
            vehicle.sourcePosition,
            vehicle.targetPosition,
            fraction
          );
          vehicle.currentPosition.latitude = newLatLng.lat();
          vehicle.currentPosition.longitude = newLatLng.lng();

          // interpolate between the source and target positions bearings using the calculated fraction
          // to get the new bearing.
          vehicle.currentPosition.bearing = lerpDegrees(
            vehicle.sourcePosition.bearing,
            vehicle.targetPosition.bearing,
            fraction
          );

          return [vehicleId, vehicle];
        })
      ),
    };
  }

  if (action.type === "setSelectedVehicle") {
    return {
      ...state,
      selectedVehicle: action.payload,
    };
  }

  if (action.type === "filterByLine") {
    return {
      ...state,
      vehicles: state.selectedVehicle.line
        ? Object.fromEntries(
            Object.entries(state.vehicles).filter(
              ([vehicleId, vehicle]) => vehicle.line === action.payload
            )
          )
        : state.vehicles,
    };
  }

  throw new Error(`Unhandled action type: ${action.type}`);
}

/*
 * Function component for the Map of the application
 */
function Map(props) {
  const [vehicleData, vehicleDataDispatch] = useReducer(vehicleDataReducer, {
    // when vehicle position updates were last received
    timestamp: 0,
    // the time interval between vehicle position updates from
    // the server
    serverUpdateInterval: 1,
    // holds the line and id of the currently selected vehicle
    selectedVehicle: {
      id: null,
      line: null,
    },
    // all vehicles
    vehicles: {},
  });
  const [currentTheme, setCurrentTheme] = useState(styles.day);
  const [currentCenter, setCurrentCenter] = useState(defaultCenter);
  const [currentRoute, setRoute] = useState(null);
  const [passengerData, setPassengerData] = useState(null);
  const [currentReservation, setCurrentReservation] = useState(null);

  const { isLoaded, loadError } = useLoadScript({
    // Reads the google-maps api_key from your locally created .env file
    googleMapsApiKey: process.env.REACT_APP_GOOGLE_MAPS_API_KEY,
  });

  const mapRef = React.useRef();
  const onMapLoad = React.useCallback((map) => {
    mapRef.current = map;
  }, []);

  // Default options of the GoogleMap component
  const options = {
    styles: currentTheme,
    disableDefaultUI: true,
    gestureHandling: "greedy",
  };

  useEffect(() => {
    const ms = 40; // milliseconds between position updates
    const updateInterval = setInterval(() => {
      vehicleDataDispatch({ type: "animate" });
    }, ms);

    return () => {
      clearInterval(updateInterval);
    };
  }, [vehicleData, vehicleData.vehicles]);

  useEffect(() => {
    vehicleDataDispatch({
      type: "setNewData",
      payload: props.realtimeData,
    });
  }, [props.realtimeData]);

  // Hook used to modify the route-data
  useEffect(() => {
    setRoute(props.route);
  }, [props.route]);

  // Hook used to modify passenger-data
  useEffect(() => {
    setPassengerData(props.passengerData);
  }, [props.passengerData]);

  // Update the position every second
  useEffect(() => {
    const interval = setInterval(() => {
      onBoundsChanged();
      //TODO: Maybe update userposition here
    }, 1000);
    return () => clearInterval(interval);
  });

  // called when the maps bounds are changed e.g. when a user drags the map
  const onBoundsChanged = () => {
    let lat = mapRef.current.getCenter().lat();
    let lng = mapRef.current.getCenter().lng();
    let radius = getBoundingSphereRadius();

    props.wsSend(JSON.stringify(geoPositionUpdateRequest(radius, lat, lng)));
  };

  // returns the radius of the maps bounding sphere in meters
  const getBoundingSphereRadius = () => {
    let center = mapRef.current.getBounds().getCenter();
    let northEast = mapRef.current.getBounds().getNorthEast();

    // return the distance along the earths surface
    return computeDistanceBetween(center, northEast);
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

  const onVehicleSelect = (vehicleId, vehicle) => {
    props.wsSend(JSON.stringify(routeRequest(vehicle.trip)));
    props.wsSend(JSON.stringify(getPassengerInfo(vehicleId)));

    vehicleDataDispatch({
      type: "setSelectedVehicle",
      payload: {
        id: vehicleId,
        line: vehicle.line,
      },
    });
    vehicleDataDispatch({
      type: "filterByLine",
      payload: vehicle.line,
    });
  };

  const onVehicleDeselect = () => {
    vehicleDataDispatch({
      type: "setSelectedVehicle",
      payload: {
        id: null,
        line: null,
      },
    });
    setRoute([]);
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
          onVehicleDeselect();
        }}
        onLoad={onMapLoad}
        onBoundsChanged={onBoundsChanged}
      >
        <MarkerClusterer options={markerClusterOptions} gridSize={20}>
          {(clusterer) =>
            Object.entries(vehicleData.vehicles).map(
              ([vehicleId, vehicle], i) => (
                <Marker
                  key={vehicleId}
                  position={{
                    lat: vehicle.currentPosition.latitude,
                    lng: vehicle.currentPosition.longitude,
                  }}
                  clusterer={clusterer}
                  onClick={() => {
                    onVehicleSelect(vehicleId, vehicle);
                  }}
                  icon={{
                    path:
                      "M25.5,8.25H23.22V3H4.82V8.25H2.5V9.53H4.82V51.34A1.67,1.67,0,0,0,6.48,53h15.1a1.65,1.65,0,0,0,1.64-1.65V9.53H25.5Z",
                    scale: vehicleId === currentReservation ? 0.7 : 0.5,
                    anchor: new window.google.maps.Point(6, 25),
                    rotation: vehicle.currentPosition.bearing,
                    fillOpacity: 1,
                    fillColor:
                      vehicleId === currentReservation ? "orange" : "green",
                  }}
                  visible={vehicleId !== setCurrentReservation}
                />
              )
            )
          }
        </MarkerClusterer>

        {currentReservation && vehicleData.vehicles[currentReservation] && (
          <Marker
            key={currentReservation}
            position={{
              lat:
                vehicleData.vehicles[currentReservation].currentPosition
                  .latitude,
              lng:
                vehicleData.vehicles[currentReservation].currentPosition
                  .longitude,
            }}
            onClick={() => {
              onVehicleSelect(
                currentReservation,
                vehicleData.vehicles[currentReservation]
              );
            }}
            icon={{
              path:
                "M25.5,8.25H23.22V3H4.82V8.25H2.5V9.53H4.82V51.34A1.67,1.67,0,0,0,6.48,53h15.1a1.65,1.65,0,0,0,1.64-1.65V9.53H25.5Z",
              scale: 0.7,
              anchor: new window.google.maps.Point(6, 25),
              rotation:
                vehicleData.vehicles[currentReservation].currentPosition
                  .bearing,
              fillOpacity: 1,
              fillColor: "orange",
            }}
          />
        )}

        {vehicleData.selectedVehicle.id &&
          vehicleData.vehicles[vehicleData.selectedVehicle.id] && (
            <InfoWindow
              position={{
                lat:
                  vehicleData.vehicles[vehicleData.selectedVehicle.id]
                    .currentPosition.latitude,
                lng:
                  vehicleData.vehicles[vehicleData.selectedVehicle.id]
                    .currentPosition.longitude,
              }}
              onCloseClick={() => {
                onVehicleDeselect();
              }}
            >
              <div>
                <p>{`Bus ${
                  vehicleData.vehicles[vehicleData.selectedVehicle.id].line
                } \n Passengers ${passengerData.passengers} / ${
                  passengerData.capacity
                }`}</p>
                {!currentReservation ? (
                  <Button
                    variant="outlined"
                    color="primary"
                    disabled={
                      passengerData.passengers === passengerData.capacity
                    }
                    onClick={() => {
                      //setReservation(true);
                      setCurrentReservation(vehicleData.selectedVehicle.id);
                      props.wsSend(
                        JSON.stringify(
                          reserveSeatRequest(vehicleData.selectedVehicle.id)
                        )
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
                      //setReservation(false);
                      setCurrentReservation(null);
                      props.wsSend(JSON.stringify(unreserveSeatRequest()));
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
            updateNavigatorGeolocation(setCoordinates);
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
          updateNavigatorGeolocation(setCoordinates);
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
