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
- Forgot to rename Dockerfile to Dockerfile.dev back
