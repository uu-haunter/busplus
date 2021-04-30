
export function routeRequest(lineNo) {
  return {
    type: "get-route-info",
    payload: {
      line: lineNo,
    },
  };
};

export function geoPositionUpdateMessage(maxDistance, lat, lng) {
  return {
    type: "geo-position-update",
    payload: {
      maxDistance: maxDistance,
      position: {
        type: "Point",
        coordinates: [lat, lng],
      },
    },
  };
}
