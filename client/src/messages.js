export function routeRequest(lineNo) {
  return {
    type: "get-route-info",
    payload: {
      line: lineNo,
    },
  };
}

export function geoPositionUpdateRequest(maxDistance, lat, lng) {
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

// Message to reserve seat
export function reserveSeatRequest(vehicleId) {
  return {
    type: "reserve-seat",
    payload: {
      descriptorId: vehicleId,
    },
  };
}

// Message to unreserve seat
export function unreserveSeatRequest() {
  return {
    type: "unreserve-seat",
  };
}

// Message to get passenger info
export function getPassengerInfo(vehicleId) {
  return {
    type: "get-passenger-info",
    payload: {
      descriptorId: vehicleId,
    },
  };
}
