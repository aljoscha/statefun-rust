# Example demonstrating state expiry and delayed invocation

This example is based on the `./examples/greeter` example, but also demonstrates
delayed invocation and state expiry.

The `seen_count` state will be expired 5 seconds after its last write, unless it was written to
again before the timeout. See the definition of `seen_count_spec()` for how this is configured.

Furthermore the `main.rs` file contains examples of delayed invocation of another function,
as well as the ability to cancel a delayed invocation through the use of a token.

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

Send this message 5 times:

```
$ curl -X PUT -H "Content-Type: application/vnd.greeter.types/UserLogin" -d '{"user_id": "1", "user_name": "Joe", "login_type": "WEB"}' localhost:8090/greeter.fns/user/1
$ curl -X PUT -H "Content-Type: application/vnd.greeter.types/UserLogin" -d '{"user_id": "1", "user_name": "Joe", "login_type": "WEB"}' localhost:8090/greeter.fns/user/1
$ curl -X PUT -H "Content-Type: application/vnd.greeter.types/UserLogin" -d '{"user_id": "1", "user_name": "Joe", "login_type": "WEB"}' localhost:8090/greeter.fns/user/1
$ curl -X PUT -H "Content-Type: application/vnd.greeter.types/UserLogin" -d '{"user_id": "1", "user_name": "Joe", "login_type": "WEB"}' localhost:8090/greeter.fns/user/1
$ curl -X PUT -H "Content-Type: application/vnd.greeter.types/UserLogin" -d '{"user_id": "1", "user_name": "Joe", "login_type": "WEB"}' localhost:8090/greeter.fns/user/1
```

The `greeter.fns/user` function will track the total number of a user's visits, and when it reaches
5 total visits it will send a delayed message to `greeter.fns/delay`.
