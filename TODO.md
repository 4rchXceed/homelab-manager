# Homelab-manager

[Repository (private for now)](https://github.com/4rchXceed/homelab-scripts)
Homeserver manager

| Type   | Langs  | Libs / SubProj | For |
| ------ | ------ | -------------- | --- |
| Server | Python | Docker         |     |

## TODO
- [/] Config-handler 🛫 2026-05-27 #project/Homelab-manager
- [/] CLI commands 🛫 2026-05-27 #project/Homelab-manager
- [x] Agent-srv communication 🛫 2026-05-27 #project/Homelab-manager ✅ 2026-05-27
- [ ] Security (RSA + certs) #project/Homelab-manager
- [ ] Backup support #project/Homelab-manager
- [ ] WebUI #project/Homelab-manager
- [x] Start/Stop/Manager docker compose (services) #project/Homelab-manager ✅ 2026-05-31
- [ ] Move services #project/Homelab-manager
- [ ] Load balancing #project/Homelab-manager
- [ ] Unittests #project/Homelab-manager 
- [ ] Performance improvements #project/Homelab-manager 
- [ ] Application repository #project/Homelab-manager  
- [ ] Allow importing other files in json config OR move to another file format (e.g. YAML) #project/Homelab-manager

## Other things
- [x] Handle when services is no longer in the config #project/Homelab-manager  ✅ 2026-06-19
- [x] Migrate every commands to send_pingpong() #project/Homelab-manager  ✅ 2026-06-19
- [x] Server when config:reload #project/Homelab-manager  ✅ 2026-06-20
- [x] Service assignment with a config file (auto-gen or manual) #project/Homelab-manager  ✅ 2026-06-20
- [x] Custom service config reload command(s): `"whenConfigUpdated": ["cmd1","cmd2"] ` #project/Homelab-manager  ✅ 2026-06-20
- [x] The config:reload is broken -> doesn't reload sometimes (generators) #project/Homelab-manager  🛫 2026-06-20 ✅ 2026-06-20
- [ ] Better way to handle when one single server doesn't respond #project/Homelab-manager
