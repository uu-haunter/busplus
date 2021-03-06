# Protocol documentation
Early version of the protocol used for communication, subject to be changed and expanded. Not all messages are implemented.

# Format of messages

Every message that is sent between the client and the server must be in the format described below. The `type` field describes what type of request/response is being sent, and the `payload` value contains whatever data is associated with that message. Important to note here is that payload can have many different types, i.e it can be an object, array or a specific value. Please look at the specification for each individual message to know what data the payload contains.

```json
{
    "type": "<unique description of request/response>",
    "payload": {
        "data": []
    }
}
```

# Error message
An error message is sent by the server if the client has sent an unknown message or bad data, if the server can't handle a request due to some reason or if the database server is down etc.
```json
{
    "type": "error",
    "payload": {
        "error_type": "SERVER_ERROR | UNKNOWN_MESSAGE | BAD_DATA | POSITION | LINE_INFO | ROUTE_INFO | RESERVE | UNRESERVE",
        "error_message": "<error message>",
    }
}
```

# Client messages
Messages that are sent from a client to the server.

### Update position
When the position or zoom on the client's map is changed, this message should be sent to the server so that only information about busses that can be seen are sent to the client.
```json
{
    "type": "geo-position-update",
    "payload": {
        "maxDistance": 1000,
        "position": {
            "type": "Point",
            "coordinates": [56.133, 13.128],
        }
    }
}
```

### Get route information
Sent to get information about a specific route. "id" can either be a line number (i.e "1", "13", "844" etc) or a trip id, in which case the response from the server will be more detailed.
```json
{
    "type": "get-route-info",
    "payload": {
        "id": "5"
    }
}
```

### Get passenger info for a bus
Sent to get information about how many passengers and capacity a bus has
```json
{
    "type": "get-passenger-info",
    "payload": {
        "descriptorId": "123456"
    }
}
```

### Reserve seat
Sent to reserve a seat on a bus with a specific id.
```json
{
    "type": "reserve-seat",
    "payload": {
        "descriptorId": "123456"
    }
}
```

### Unreserve seat
Sent to unreserve a seat on a bus with a specific id.
```json
{
    "type": "unreserve-seat"
}
```

# Server messages
Messages that are sent from the server to clients.

### Vehicle positions
Sends the information of all vehicles set by `geo-position-update`. 
> Note that `trip_id` is not always defined so be sure to check if it is null or an actual value.
```json
{
    "type": "vehicle-positions",
    "payload": {
        "timestamp": 111111,
        "vehicles": [
            {
                "descriptor_id": "123456",
                "trip_id": "123456",
                "line": 5,
                "position": {
                    "latitude": 59,
                    "longitude": 16,
                    "bearing": 180,
                    "speed": 30
                }
            },
            ...
        ]
    }
}
```

### Passenger information
Get information about how many passenger and capacity a bus has
```json
{
    "type": "passenger-info",
    "payload": {
        "passengers": 13,
        "capacity": 30,
    }
} 
```

### Route information
Get the coordinates for a specific route.
```json
{
    "type": "route-info",
    "payload": {
        "timestamp": 111111,
        "route": [
            {"lat": 37.772, "lng": -122.214},
            {"lat": 21.291, "lng": -157.821},
            {"lat": -18.142, "lng": 178.431},
            {"lat": -27.467, "lng": 153.027}
        ]
    }
}
```