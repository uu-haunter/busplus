import React from "react";
import { useState, useEffect } from "react";
import {
  GoogleMap,
  useLoadScript,
  Marker,
  InfoWindow,
} from "@react-google-maps/api";
import { computeDistanceBetween } from 'spherical-geometry-js';
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

	useEffect(() => {
		setRealtimeData(props.realtimeData);
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
				"radius": radius,
				"position": {
					"latitude": lat,
					"longitude": lng
				}
			}
		};

		props.wsSend(JSON.stringify(message);
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
    googleMapsApiKey: "AIzaSyDm354e4VJMSH5rVD93KcgEoKXXlSeTCnE",
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
        onLoad={onMapLoad}
				onBoundsChanged={onBoundsChanged}
      >
        {realtimeData.map((obj) => (
          <Marker
            key={obj.id}
            position={{
              lat: obj.position.latitude,
              lng: obj.position.longitude,
            }}
            onClick={() => {
              setSelectedMarker(obj);
            }}
						rotation={obj.position.bearing}
          >
          </Marker>
        ))}
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
