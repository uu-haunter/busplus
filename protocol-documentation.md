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

### Get line information
Sent to get information about a specific line.
> Not implemented
```json
{
    "type": "get-line-info",
    "payload": {
        "line": 5
    }
}
```

### Reserve seat
Sent to reserve a seat on a bus with a specific id.
> Not implemented
```json
{
    "type": "reserve-seat",
    "payload": {
        "id": 123456
    }
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
                "capacity": 80,
                "passengers": 30,
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

### Line information
Get the information from a specific line.
>  Not implemented
```json
{
	"type": "line-info",
	"payload": {
		"timestamp": 111111,
		"line" : 5,
    	"vehicles": 10,
        "stops": [
			{
				"name": "Centralstationen",
                "lines" : [5, 11, 14],
				"position": {
                    "type": "Point",
                    "coordinates": [56.133, 13.128],
                }
			},
			...
		]
	}
}
```

### Stop information
Get the information from a specific stop.
>  Not implemented
```json
{
	"type": "stop-info",
	"payload": {
        "name": "Centralstationen",
		"lines" : [5, 11, 14],
        "position": {
            "type": "Point",
            "coordinates": [56.133, 13.128],
        } 
	}
}
```