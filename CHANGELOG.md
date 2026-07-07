# Changelog

## How-it-works
As title: the *LAST* commit hash ! NOT THE CURRENT

... (before there was no changelog)
## LAST: 7f095b16d8719ed0a2aa841ce8e4de47df345af0 - Minor bugfixes + added IP-reload when deleting/unassign a service
Removed support for raiseNotFound, as this will make everything crash.
Added "cleanup" function for the variable providers + switch to cmd_context for cli_frontend
Added an entirely new configuration file: emergency procedures: allows actions to be executed when specific events are triggered

### Commit message: `- raiseNotFound removed on ip-type var-provider; + cleanup function for variable providers; ~ switched to cmd_context for cli_frontend [variable providers]; + emergency procedures system: allows actions to be executed when specific events are triggered`

## LAST: 0fbc8728a3760ccaee6fa453a397a576f612842b
Added support for python3.12 (required for py-spy profiling)
Added py-spy profiling script
Fixed a bug in server/src/protocol/agent.py:200 (After the fix, CPU usage decreased from 150% to 0.5%)

### Commit message: `+ support for python3.12; + py-spy profiling script; ~ fixed CPU usage bug in agent.py`

### Bugfixes childs commits:
- Forgot to rename Dockerfile to Dockerfile.dev back (LAST: 1864c3ab22a6e60bec65221bee355b6357ce5547)

## LAST: e03d0fdc515023de5771fa38262057f2dda61a44
~ Fixed 2 bugs: docker down+up instead of restart, and other small bugfixes
+ Added a new command: `var:set` to set a variable value from the CLI
+ Added a user variable dependency, so when a variable is changed, the dependent config will be regenerated automatically
+ You can now write _import in the json config file, which will import a dict or list from another json file (! supports http/https urls, disable with CFG_NO_WEB_IMPORT env var)
~ Migrated config files to jsonc format

### Commit message: `Fixed 2 bugs, added var:set command, added user variable dependencies, added _import support in json config files, migrated config files to jsonc format. See CHANGELOG.md for more details.`

## LAST: ace24868318d1d8543a81149610c314a0d759b25
~ Fixed a bug in the previous commit, _imports did not work at all (yeah, I know, I should have tested it before committing)
+ Added *some* (not all of them) tests. (YAY :) ). There: 1_test-service-basic 2_test-generator-1 3_test-generator-2 4_test-uservar-env
+ Added test base (py scripts + tearup shell script)
+ Sleepy rn it's 2 am tired of doing tests :(

### Commit message: `Fixed a bug in the previous commit, _imports did not work at all; Added some tests; Added test base;`

## LAST: 08d3d467182dfb959d3fabf9701493b164247277
~ Improved A LOT the test script
+ Added a .agent_state (STARTING | RUNNING) file to the agent
~ Fixed some bugs
+ Added tests:
  - test uservar (with input)
  - test with multiple servers (basic)
  - test with multiple servers (with ip variable)
  - test with multiple servers (with ip variable + changing ip so does the config)

### Commit message: `Improved test script; Added .agent_state file to agent; Fixed some bugs; Added tests for uservar and multiple servers. See CHANGELOG.md for more details.`

## LAST: 64c4769d179dbd21b3024212989edb482eb54ba6
~ Fixed a bug with config:reload (wasn't using jsonc + imports)
+ Added json5 dependency to tests
+ Added tests (for commands):
  - config:reload
  - config:regen
  - config:runtime (full test of runtime.json config)
~ Bugfixes + code improvements

### Commit message: `Fixed a bug with config:reload; Added json5 dependency to tests; Added tests for config:reload, config:regen, and config:runtime, bugfixes & code improvements. See CHANGELOG.md for more details.`

## LAST: 14403654bafc770816aeab2038efb86d6b2b85cf
~ Fixed a bug with list_services
- Removed unused .gitkeep files
+ Added tests (for commands):
  - exec:raw service <agent> restart
  - services:sync
  - config:emergency_proc reload (tests the emergency procedures system) !! only the shell action, but the 2 listeners
~ Improved test script
+ Switched server's docker image to python (*debian*)
~ Few other improvements / bugfixes / code improvements
! Finished tests (for now)

### Commit message: `~ Fixed a bug with list_services - Removed unused .gitkeep files + Added tests (for commands):   - exec:raw service <agent> restart   - services:sync   - config:emergency_proc reload (tests the emergency procedures system) !! only the shell action, but the 2 listeners ~ Improved test script + Switched server's docker image to python (*debian*) ~ Few other improvements / bugfixes / code improvements`

## LAST: 371e0d77af6535b7b4dcee50631a53be52f03012
+ Switched protocol from Socket to SSL Sockets
+ Added new client check: reverse_api_key, so the agent can verify the server's identity
+ Added new certificate file (server.crt) and key file (server.key) for the server (mandatory, and server.crt needs to be passed to the client)
+ Added new commands:
  - service:build -> `docker compose build` for a service
  - security:regen-cert -> regenerates the SSL certificate
+ Added new tests:
  - 16: test_service_build
  - 17: test_security_regen_cert
~ Small bugfixes
~ Updated diagrams to match new DB architecture
+ Added fileserverAuth / fileserver_auth config for client+server so the file server can be behind a password (TODO: make the fileserver https)


### Tests report:
Test 17 passed
Passed: 16/17
Failed: 1/17
Some tests failed
#### After bugfix:
Test 15 passed
Passed: 1/1
All tests passed

### Commit message: `New SSL security for socket communication, added certificate generation and client's server identity verification, added new commands: service:build, security:regen-cert, added new tests: test_service_build, test_security_regen_cert, small bugfixes, updated diagrams, Added fileserverAuth / fileserver_auth config for client+server, so the file server can be behind a password`

## LAST: 45febb8d41bbcdc299f5cb5581a7960e249d3687
~ Switched the file server to use HTTPS (via a reverse proxy written in pure Python)
~ Fixed a bug on cert check
- Removed static TODO list, and added a link to my Nextcloud shared folder

### Commit message: `~ Switched the file server to use HTTPS (via a reverse proxy written in pure Python) ~ Fixed a bug on cert check - Removed static TODO list, and added a link to my Nextcloud shared folder`

## LAST: a49e5aa652eb5e0e8fd4337a9974027d2e502740
~ Fixed file server, it was broken (the reverse proxy was not working properly)
+ Added new config: backups (in services, in config.jsonc)
+ Added new config: backupAssignments (in runtime.jsonc)
+ Added new backup system, which allows to backup files from the server to an agent
+ Added new config: storages (in servers, in config.jsonc)
+ Added new test:
  - test-full-backup (tests the backup system) on "full" mode

### Disclaimer
The backup system is not finished. I'm going to work on it tomorrow. Here what's not implemented yet:
- Incremental backups TESTS (implemented, never tested, probably broken)
- Restoring
- Commands
- A few other things

### Commit message: `~ Fixed file server, it was broken (the reverse proxy was not working properly) + Added new config: backups (in services, in config.jsonc) + Added new config: backupAssignments (in runtime.jsonc) + Added new backup system, which allows to backup files from the server to an agent + Added new config: storages (in servers, in config.jsonc) + Added new test: - test-full-backup (tests the backup system) on "full" mode`

## LAST: d99cdb307f0949980f3e60f20b7c9731efd13942
+ Added tests for incremental backups (test-incremental-backup)
+ Fixed A LOT of bugs in the backup system (incremental backups, full backups, etc)
+ Rewrote a lot of the backup system code, now it's way easier to read and understand
+ Added new commaand: `backup:create` to force-create a backup (full or incremental)
+ Added new command: `backup:restore` to restore a backup (full or incremental)
+ Added new command: `backup:list` to list all backups from a storage (full and incremental)
+ Added 4 tests: 
  - 21_test-full-backup-restore
  - 22_test-incremental-backup-restore
  - 19_test-incr-backup
  - 20_test-incr-backup-2
~ Few other things

### Commit message: `Added tests: 21_test-full-backup-restore, 22_test-incremental-backup-restore, 19_test-incr-backup, 20_test-incr-backup-2. Fixed A LOT of bugs in the backup system. Added new commands: backup:create, backup:restore, backup:list. Rewrote a big part of the backup system code. See CHANGELOG.md for more details.`

## LAST: 3bab629f99a1ed17d7f935d3c8745819e6ae6b82
+ New option: noBackupOnCreation
~ Bugfixes in backup
~ Tested (in private) a backup of entire nextcloud instance (with ~1GB of data). Worked
+ Added with_size option to backup:list command, so it will show the size of each backup
~ Fixed a bug in the config file generation (see before_regenerate in server/src/services/config_file.py)
~ Some other small bugfixes

### Backups TODO:
- Files permission
- Sync "backup" type

### Commit message: `New backup cfg option + bugfixes in backup system + with_size option t backup:osit + Fixed a bug in config file generation + Some other small bugfixes. See CHANGELOG.md for more details.`

## LAST: d1bf372c49032dca381e733808aa6bde079994fd
+ New config type (runtime.jsonc): syncs: allows to sync datas from a service to an agent
+ New tests: 23_test-service-sync, 24_test-service-full-sync
- Removed tests/configs/backups folder (was test datas)
+ Fixed some bugs.
+ New command: sync:full -> forces a full sync

### Disclaimer
The sync system is here, but I still need to implement the core functionality: when assigning a service to a new agent, it should automatically sync the data from the service to the agent. This is not implemented yet

### Commit message: `New runtime config type: syncs, 2 new tests: 23_test-service-sync, 24_test-service-full-sync, new command: sync:full, fixed some bugs. See CHANGELOG.md for more details.`
