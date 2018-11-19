# Red or Black game websocket server
This is a multiplayer red or black drinking game that uses websockets and is built in rust. This is just the webserver, the frontend can be found [here](https://github.com/mickjohn/red_or_black_frontend)

## Getting started
This will show you how to get this project up and running.

### Prerequisites
You need to have rust installed, and you may need install gcc as well. You can get rust [here](https://rustup.rs/)

### Building and running locally
Build and run the project with
```
cargo run --release
```
The server should now be running and ready to accept connections.

The address and port that the server listens on can be configured using two environment variables, `RED_OR_BLACK_WEBSERVER_PORT` & `RED_OR_BLACK_WEBSERVER_PORT`. e.g.

```
export RED_OR_BLACK_WEBSERVER_ADDRESS="0.0.0.0"
export RED_OR_BLACK_WEBSERVER_PORT=12345
cargo run --release
```

After the executable has been built the docker image can be built using:
```
docker build -t red_or_black_server .
```
## Licence
This project is licensed under the MIT Licence - see the LICENCE.txt file for details
