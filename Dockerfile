FROM ubuntu:18.04
EXPOSE 9000

ENV RED_OR_BLACK_WEBSERVER_ADDRESS 0.0.0.0
ENV RED_OR_BLACK_WEBSERVER_PORT 9000

COPY target/release/websocket_red_or_black /
CMD ./websocket_red_or_black
# CMD /bin/bash
