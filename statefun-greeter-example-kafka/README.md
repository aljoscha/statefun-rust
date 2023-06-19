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
docker-compose up -d
```

Then, to see the example in actions, see what comes out of the topic
`greetings`:

```
docker-compose logs -f datagen
```

