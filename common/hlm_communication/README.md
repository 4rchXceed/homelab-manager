# Homelab-manager Communication library
This library is used to handle communication between the homelab-manager server and its agents.
It's a library since both the server and the agents need to use it.

## Communication protocol
The communication protocol is based on basic SSL sockets, with basic JSON data exchange.
This library provides a simple trait and runner to create communication socket, implement API keys, etc.

## Example:
See example -> A simple example of how to use the library to create a server and an agent that can communicate with each other.
