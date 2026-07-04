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
