# A Simple Greeter Example

This is a simple example that runs two stateful functions that accept requests
from a Kafka ingress, and then respond by sending greeting responses to a Kafka
egress. It demonstrates the primitive building blocks of a Stateful Functions
applications, such as ingresses, handling state in functions, and sending
messages to egresses.

The example consists of two stateful functions, a `greeter` and a `relay` that
show how messages can be passed between stateful functions. All the `relay`
does is package messages to send them to the Kafka Egress.

## Running the example

To run the example:

```
docker-compose up --build
```

## Play around!

The greeter application allows you to do the following actions:

* Create a greeting for a user via sending a `UserLogin` message to the `user` function

In order to send messages to the Stateful Functions application you can run:

```
$ curl -X PUT -H "Content-Type: application/vnd.greeter.types/UserLogin" -d '{"user_id": "1", "user_name": "Joe", "login_type": "WEB"}' localhost:8090/greeter.fns/user/1
```

You can take a look at what messages are being sent to the Playground egress:

```
$ curl -X GET localhost:8091/greetings
```
