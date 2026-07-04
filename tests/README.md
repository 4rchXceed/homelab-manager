# Homelab-manager tests

This folder provides a testing suite for the Homelab-manager project. The tests are designed to ensure that the various components of the project function as expected.
!! This does not test any edge cases or specific configurations, but rather the general functionality of the project. !!

## Tests (TODO):
- [X]: Test starting a basic service
- [X]: Test generating a configuration file
- [X]: Test generating a configuration file (empty at first)
- [X]: Test generating a configuration file with a variable (passed by ENV)
- [X]: Test generating a configuration file with a variable (passed by INPUT)
- [X]: Test a multiple-server setup
- [X]: Test a multiple-server setup with configuration mentionning the other server's IP
- [X]: Test a multiple-server setup with configuration mentionning the other server's IP, change it, and check if the config is updated
- [ ]: Config commands:
  - [X]: config:regen
  - [X]: config:reload
  - [X]: config:runtime
  - [ ]: config:sync
  - [ ]: config:emergency_proc
  - [ ]: exec:raw <agent> restart <service>
  - [ ]: service:unassign
  - [ ]: services:sync
  - [ ]: var:set
- [ ]: Emergency procedures
