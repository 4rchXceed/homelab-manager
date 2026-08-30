# Homelab-manager Communication library
This library is used to handle communication between the homelab-manager server and its agents.
It's a library since both the server and the agents need to use it.

## Communication protocol
This library is made to create custom communication protocol between the server and the agents. The protocol is based on TCP *SSL* sockets, and uses a customized message format.

## Example:
See example -> A simple example of how to use the library to create a server and an agent that can communicate with each other.

## TODO
- [x] Basic implementation of the socket communication protocol.
- [x] Message builder
- [ ] Documentation
- [ ] Probably: Add a way to "sync" the message format between the server and the agents, maybe via a file format
