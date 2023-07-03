# A Simple Greeter Example

This is an example based on the [Java Playground Example](https://github.com/apache/flink-statefun-playground/tree/1ff449204b367e6dd0d0818ca76a5283890ce2c5/java/greeter)
utilizing custom ingress and egress types as defined in the playground.

For a more real-world example utilizing vanilla Kafka instances, please take a look at
`./examples/kafka` instead in the project root.

## Running the example

To run the example:

```
docker-compose up --build
```

If you have problems running docker-compose try to build manually and enable console logging:

```
docker-compose build --progress plain
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
