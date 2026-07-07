import subprocess
import signal
import json5
import json
import traceback
import threading
import os
import shutil
import sys
import time
import unittest
from homelab_manager import HomelabManagerInstance
from result_srv import ResultServer, set_test, is_success, get_counter

server = None
kill_switch = False
def launch_service(service: str, instance: HomelabManagerInstance, inputs: list[str] = [], to: str="agent01"):
    print(f"Starting service {service}... (may take some time) [threaded]")
    while not kill_switch and not is_success():
        try:
            instance.send_command(f"service:assign {service} {to}", inputs=inputs)
        except BrokenPipeError:
            pass
        time.sleep(1)

killswitch = False

class TestHomelabManager():
    def __init__(self) -> None:
        self.nbr_success = 0

    def is_http_server_running(self, ip: str, port: int) -> bool:
        import socket
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(1)
            try:
                sock.connect((ip, port))
                return True
            except (socket.timeout, ConnectionRefusedError):
                return False
    def read_file(self, path: str) -> str:
        try:
            with open(path, "r") as f:
                return f.read()
        except FileNotFoundError:
            return ""

    # Tests starting services and checking if they report success to the result server.
    def test_service_basic(self):
        nbr = "1"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        service = "1_test-service-basic"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service basic test failed"
        th.join(0.1)
        self.ok(nbr)

    # Tests generating a config with a generator
    def test_generator_1(self):
        nbr = "2"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "2_test-generator-1"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 1 test failed"
        th.join(0.1)
        self.ok(nbr)

    # Tests generating a config with a generator (other type)
    def test_generator_2(self):
        nbr = "3"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "3_test-generator-2"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        th.join(0.1)
        self.ok(nbr)

    # Tests passing a user variable to a service's config with a generator
    def test_uservar_1(self):
        nbr = "4"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"], "USERVAR_TESTVAR=ok")
        print("Starting services... (may take some time)")
        service = "4_test-uservar-env"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service uservar 1 test failed"
        th.join(0.1)
        self.ok(nbr)


    # Tests passing a user variable to a service's config with a generator (with user input from CLI)
    def test_uservar_2(self):
        nbr = "5"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "5_test-uservar-input"
        th = threading.Thread(target=launch_service, args=(service, instance, ["ok"]))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service uservar 2 test failed"
        th.join(0.1)
        self.ok(nbr)

    # Tests a multiple servers setup with a service on each server, and checks if they can communicate with each other.
    def test_multiple_servers_basic(self):
        nbr = "6"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"])
        print("Starting services... (may take some time)")
        service2 = "6_test-multservers-basic-srv02"
        service1 = "6_test-multservers-basic-srv01"
        instance.send_command(f"service:assign {service2} agent02")
        th = threading.Thread(target=launch_service, args=(service1, instance, [], "agent01"))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        th.join(0.1)
        self.ok(nbr)

    # Tests a multiple servers setup with a service on each server, and checks if they can generate a config with the other server's IP address.
    def test_multiple_servers_ip(self):
        nbr = "7"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"])
        print("Starting services... (may take some time)")
        service2 = "7_test-multservers-ip-srv02"
        service1 = "7_test-multservers-ip-srv01"
        instance.send_command(f"service:assign {service2} agent02")
        th = threading.Thread(target=launch_service, args=(service1, instance, [], "agent01"))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        print("Removing old services...")
        instance.send_command(f"service:unassign {service1}")
        instance.send_command(f"service:unassign {service2}")
        th.join(0.1)
        self.ok(nbr)

    # Same as test_multiple_servers_ip, but with a changes the IP address of one of the servers in the middle of the test, to check if the config generation is updated correctly.
    def test_multiple_servers_ip_with_change(self):
        nbr = "8"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"])
        print("Starting services... (may take some time)")
        service2 = "8_test-multservers-ip-change-srv02"
        service1 = "8_test-multservers-ip-change-srv01"
        instance.send_command(f"service:assign {service2} agent02")
        instance.send_command(f"service:assign {service1} agent01")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        print("Removing old services...")
        instance.send_command(f"service:unassign {service1}")
        instance.send_command(f"service:unassign {service2}")
        self.ok(nbr)

    # Tests updating the configuration and reloading it
    def test_reload_config(self):
        nbr = "9"
        set_test(nbr, 2, multiple_obj=True)
        if os.path.exists("configs/tests/9.jsonc.donottouch.internal"):
            os.unlink("configs/tests/9.jsonc.donottouch.internal")
        shutil.copyfile("configs/tests/9_bak.jsonc", "configs/tests/9.jsonc")
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "9_test-reload-config-command"
        instance.send_command(f"service:assign {service} agent01")
        print("Waiting for first test result.", end="", flush=True)
        s = time.time()
        while time.time() - s < 120 and not get_counter() == 1:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        shutil.copyfile("configs/tests/9_2_bak.jsonc", "configs/tests/9.jsonc")
        instance.send_command("config:reload")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        print("Removing old services...")
        instance.send_command(f"service:unassign {service}")
        self.ok(nbr)


    # Tests regenerating the configuration for a service and reloading it
    def test_regen_config(self):
        nbr = "10"
        set_test(nbr, 2, multiple_obj=True)
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "10_test-config-regen-command"
        instance.send_command(f"service:assign {service} agent01")
        print("Waiting for first test result.", end="", flush=True)
        s = time.time()
        while time.time() - s < 120 and not get_counter() == 1:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        instance.send_command(f"config:regen {service}")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        print("Removing old services...")
        instance.send_command(f"service:unassign {service}")
        self.ok(nbr)


    # Tests the runtime.jsonc configuration file
    def test_runtime_config(self):
        nbr = "11"
        shutil.copyfile("configs/runtime/11_bak.jsonc", "configs/runtime/11.jsonc")
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/11.jsonc")
        print("Starting services... (may take some time)")
        instance.send_command(f"config:runtime reload")
        print("Waiting for FIRST test result.", end="", flush=True)
        s = time.time()
        while time.time() - s < 120 and not self.is_http_server_running("192.168.239.10", 5005):
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert self.is_http_server_running("192.168.239.10", 5005)
        shutil.copyfile("configs/runtime/11_2_bak.jsonc", "configs/runtime/11.jsonc")
        instance.send_command("config:runtime reload")
        s = time.time()
        print("Waiting for SECOND test result.", end="", flush=True)
        while time.time() - s < 120 and self.read_file("tmp/agent02/services/11_test-config-runtime/status").strip() != "OK":
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert self.read_file("tmp/agent02/services/11_test-config-runtime/status").strip() == "OK"
        shutil.copyfile("configs/runtime/11_3_bak.jsonc", "configs/runtime/11.jsonc")
        print("Reloading runtime config...")
        instance.send_command("config:runtime reload")
        s = time.time()
        print("Waiting for THIRD test result.", end="", flush=True)
        while time.time() - s < 120 and self.is_http_server_running("192.168.239.10", 5005):
            time.sleep(1)
            print(".", end="", flush=True)
            print("ok")
        assert not self.is_http_server_running("192.168.239.10", 5005)
        print("Removing old services...")
        instance.send_command(f"service:unassign 11_test-config-runtime")
        print("Re-assigning service...")
        instance.send_command(f"service:assign 11_test-config-runtime agent01")
        time.sleep(1)
        print("Dumping config...")
        instance.send_command(f"config:runtime dump")
        time.sleep(1)
        print("Checking dumped config...")
        with open("configs/runtime/11.jsonc", "r") as f:
            content = f.read()
        config_ok = False
        dumped_config: dict = json5.loads(content).get("assignments", {})
        if len(dumped_config.keys()) == 1 and "11_test-config-runtime" in dumped_config and dumped_config["11_test-config-runtime"] == "agent01":
            config_ok = True
        assert config_ok, f"Dumped config is incorrect: {dumped_config}"
        print("ok")
        self.ok(nbr)

    # Tests the exec_raw command with a service that restarts itself
    def test_exec_raw_restart(self):
        nbr = "12"
        set_test(nbr, 2)
        instance = HomelabManagerInstance(nbr,["agent01"])
        service = "12_test-exec-raw-restart"
        instance.send_command(f"service:assign {service} agent01")
        s = time.time()
        instance.send_command(f"exec:raw service agent01 restart {service}")
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service exec_raw_restart test failed"
        self.ok(nbr)

    # Tests the services:sync command.
    def test_services_sync(self):
        nbr = "13"
        instance = HomelabManagerInstance(nbr,["agent01"])
        service = "13_test-services-sync"
        instance.send_command(f"service:assign {service} agent01")
        s = time.time()
        instance.send_command(f"exec:raw service agent01 stop {service}")
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and self.is_http_server_running("192.168.239.10", 5006):
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert not self.is_http_server_running("192.168.239.10", 5006), "Test test_services_sync test failed"
        # allow_actions => Allow starting / stopping services
        # Btw: the database state has a priority over the actual state of the services.
        instance.send_command(f"services:sync allow_actions") # Since we did not stopped it with the "normal" way (service:unassign), the service is still marked as "running" in the agent's state, so we need to sync the state with the actual state of the services (aka. start the service again)
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not self.is_http_server_running("192.168.239.10", 5006):
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert self.is_http_server_running("192.168.239.10", 5006), "Test test_services_sync test failed"
        self.ok(nbr)


    # Tests passing a user variable to a service's config with a generator
    def test_var_set(self):
        nbr = "14"
        set_test(nbr, 2, multiple_obj=True)
        instance = HomelabManagerInstance(nbr,["agent01"], "USERVAR_TESTVAR=ok/1")
        print("Starting services... (may take some time)")
        service = "14_test-var-set"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not get_counter() == 1:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert get_counter() == 1, "Test var set test failed"
        instance.send_command(f"var:set testVar ok/2")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not get_counter() == 2:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert get_counter() == 2, "Test var set test failed"
        th.join(0.1)
        self.ok(nbr)


    # Tests the emergency_proc configuration file and the config:emergency_proc reload command
    def test_emergency_proc(self):
        nbr = "15"
        if os.path.exists("configs/emergency_proc/15.jsonc"):
            os.unlink("configs/emergency_proc/15.jsonc")
        shutil.copyfile("configs/emergency_proc/15_bak.jsonc", "configs/emergency_proc/15.jsonc")
        set_test(nbr, 2, multiple_obj=True)
        instance = HomelabManagerInstance(nbr,["agent01"], "EMERGENCY_CONFIG_FILE=../tests/configs/emergency_proc/15.jsonc", do_not_connect_agents=True)
        print("Reloading config...")
        instance.send_command(f"config:reload")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not get_counter() == 1:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert get_counter() == 1, "Test emergency proc test failed"
        shutil.copyfile("configs/emergency_proc/15_2_bak.jsonc", "configs/emergency_proc/15.jsonc")
        instance.send_command(f"config:emergency_proc reload")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not get_counter() == 2:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert get_counter() == 2, "Test emergency proc test failed"
        self.ok(nbr)


    # Tests the service:build command
    def test_service_build(self):
        nbr = "16"
        set_test(nbr, 2, multiple_obj=True)
        if os.path.exists("./services/16_test-service-build/Dockerfile"):
            os.unlink("./services/16_test-service-build/Dockerfile")
        shutil.copyfile("./services/16_test-service-build/Dockerfile_bak", "./services/16_test-service-build/Dockerfile")
        instance = HomelabManagerInstance(nbr,["agent01"])
        service = "16_test-service-build"
        instance.send_command(f"service:build {service}")
        instance.send_command(f"service:assign {service} agent01")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not get_counter() == 1:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert get_counter() == 1, "Test service:build part 1 failed"
        shutil.copyfile("./services/16_test-service-build/Dockerfile_bak_2", "tmp/agent01/services/16_test-service-build/Dockerfile")
        instance.send_command(f"service:build {service}")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not get_counter() == 2:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert get_counter() == 2, "Test service:build part 2 failed"
        self.ok(nbr)


    # Tests the security:regen-cert
    def test_security_regen_cert(self):
        nbr = "17"
        instance = HomelabManagerInstance(nbr,["agent01"], do_not_connect_agents=True)
        pid = instance.start_client("agent01")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not instance.send_command(f"exec:raw service agent01 list").strip().endswith("OK"):
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert time.time() - s < 120, "Test security:regen-cert failed"
        instance.send_command(f"security:regen-cert")
        instance.restart_server()
        os.kill(pid, signal.SIGTERM)
        instance.start_client("agent01", do_not_copy_cert=True)
        assert instance.agent_started(os.path.join("tmp", "agent01", ".agent_state")) == 1, "Agent did not crash after security:regen-cert but no cert update on clients"
        self.ok(nbr)

    # Tests the full backup (auto)
    def test_backup_full(self):
        nbr = "18"
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/18.jsonc")
        instance.send_command("config:runtime reload")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 120 and not self.folder_size("tmp/agent01/test-storage") == 25:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert self.folder_size("tmp/agent01/test-storage") == 25, "Test backup full failed: folder size is not 25 bytes"
        self.ok(nbr)

    # Tests the incremental backup (with commands)
    def test_incremental_backup_1(self):
        nbr = "19"
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/19.jsonc BACKUP_DUMP=yes")
        print("Reloading runtime config...")
        instance.send_command("config:runtime reload no_backup_check")
        service = "19_test-incr-backup"
        print("Backup: stage init")
        time.sleep(1) # 2 backups are not supposed to be at the same second, since it's classed by timestamps. You cannot define an auto backup at less than 1 minute of interval, so it's fine
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage1.sh", shell=True, check=True, cwd="tmp/agent02/services/19_test-incr-backup")
        print("Backup: stage 1")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage2.sh", shell=True, check=True, cwd="tmp/agent02/services/19_test-incr-backup")
        print("Backup: stage 2")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage3.sh", shell=True, check=True, cwd="tmp/agent02/services/19_test-incr-backup")
        print("Backup: stage 3")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 5 and not self.folder_size("tmp/agent01/test-storage", except_files=["index.json"]) == 2097164:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        size = self.folder_size("tmp/agent01/test-storage", except_files=["index.json"])
        assert size == 2097164, f"Test incremental backup failed: folder size is {size}, not 2097164 bytes"
        self.ok(nbr)

    # Tests the incremental backup (2, more "edge" cases) (with commands)
    def test_incremental_backup_2(self):
        nbr = "20"
        if os.path.exists("configs/tests/20.jsonc"):
            os.unlink("configs/tests/20.jsonc")
        shutil.copyfile("configs/tests/20_bak.jsonc", "configs/tests/20.jsonc")
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/20.jsonc BACKUP_DUMP=yes")
        print("Reloading runtime config...")
        instance.send_command("config:runtime reload no_backup_check")
        service = "20_test-incr-backup-2"
        print("Backup: stage init")
        time.sleep(1) # 2 backups are not supposed to be at the same second, since it's classed by timestamps. You cannot define an auto backup at less than 1 minute of interval, so it's fine
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage1.sh", shell=True, check=True, cwd="tmp/agent02/services/20_test-incr-backup-2")
        print("Backup: stage 1")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage2.sh", shell=True, check=True, cwd="tmp/agent02/services/20_test-incr-backup-2")
        print("Backup: stage 2")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage3.sh", shell=True, check=True, cwd="tmp/agent02/services/20_test-incr-backup-2")
        print("Backup: stage 3")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 5 and not self.folder_size("tmp/agent01/test-storage", except_files=["index.json"]) == 38:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        size = self.folder_size("tmp/agent01/test-storage", except_files=["index.json"])
        assert size == 38, f"Test incremental backup failed: folder size is {size}, not 38 bytes"
        if os.path.exists("configs/tests/20.jsonc"):
            os.unlink("configs/tests/20.jsonc")
        shutil.copyfile("configs/tests/20_bak_2.jsonc", "configs/tests/20.jsonc")
        print("Reloading config...")
        instance.send_command("config:reload")
        print("Backup: stage 4")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 5 and not self.folder_size("tmp/agent01/test-storage", except_files=["index.json"]) == 10:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        size = self.folder_size("tmp/agent01/test-storage", except_files=["index.json"])
        assert size == 10, f"Test incremental backup failed: folder size is {size}, not 10 bytes"
        self.ok(nbr)

    # Tests the full backup and restore (with commands)
    def test_full_backup_restore(self):
        nbr = "21"
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/21.jsonc BACKUP_DUMP=yes")
        print("Reloading runtime config...")
        instance.send_command("config:runtime reload no_backup_check")
        service = "21_test-full-backup-restore"
        print("Backup: stage init")
        time.sleep(1) # 2 backups are not supposed to be at the same second, since it's classed by timestamps. You cannot define an auto backup at less than 1 minute of interval, so it's fine
        instance.send_command(f"backup:create {service} full_bak")
        subprocess.run("./create_datas_stage1.sh", shell=True, check=True, cwd="tmp/agent02/services/21_test-full-backup-restore")
        print("Backup: stage 1")
        time.sleep(1)
        instance.send_command(f"backup:create {service} full_bak")
        subprocess.run("./create_datas_stage2.sh", shell=True, check=True, cwd="tmp/agent02/services/21_test-full-backup-restore")
        print("Backup: stage 2")
        time.sleep(1)
        instance.send_command(f"backup:create {service} full_bak")
        all_baks = os.listdir("tmp/agent01/test-storage/21_test-full-backup-restore/full/")
        all_baks = [bak for bak in all_baks if bak != "index.json" and bak != "base"]
        assert len(all_baks) > 0, "No incremental backups found"
        all_baks = sorted(all_baks, key=lambda x: int(x))
        print(f"Found backups: {all_baks}")
        first = all_baks[0]
        print(f"Restoring first backup: {first}")
        instance.send_command(f"backup:restore {service} full_bak agent01 test-storage {first}")
        print("Backup: stage 3")
        time.sleep(1)
        instance.send_command(f"backup:create {service} full_bak")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 5 and not self.folder_size("tmp/agent01/test-storage", except_files=["index.json"]) == 7339991:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        size = self.folder_size("tmp/agent01/test-storage", except_files=["index.json"])
        assert size == 7339991, f"Test incremental backup failed: folder size is {size}, not 7339991 bytes"
        self.ok(nbr)

    # Tests the incremental backup and restore (with commands)
    def test_incremental_backup_restore(self):
        nbr = "22"
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/22.jsonc BACKUP_DUMP=yes")
        print("Reloading runtime config...")
        instance.send_command("config:runtime reload no_backup_check")
        service = "22_test-incremental-backup-restore"
        print("Backup: stage init")
        time.sleep(1) # 2 backups are not supposed to be at the same second, since it's classed by timestamps. You cannot define an auto backup at less than 1 minute of interval, so it's fine
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage1.sh", shell=True, check=True, cwd="tmp/agent02/services/22_test-incremental-backup-restore")
        print("Backup: stage 1")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        subprocess.run("./create_datas_stage2.sh", shell=True, check=True, cwd="tmp/agent02/services/22_test-incremental-backup-restore")
        print("Backup: stage 2")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        all_baks = os.listdir("tmp/agent01/test-storage/22_test-incremental-backup-restore/incremental/")
        all_baks = [bak for bak in all_baks if bak != "index.json" and bak != "base"]
        assert len(all_baks) > 0, "No incremental backups found"
        all_baks = sorted(all_baks, key=lambda x: int(x))
        print(f"Found backups: {all_baks}")
        first = all_baks[0]
        print(f"Restoring first backup: {first}")
        instance.send_command(f"backup:restore {service} incr_bak agent01 test-storage {first}")
        print("Backup: stage 3")
        time.sleep(1)
        instance.send_command(f"backup:create {service} incr_bak")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 5 and not self.folder_size("tmp/agent02/services/22_test-incremental-backup-restore/data", except_files=["index.json"]) == 2097138:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        size = self.folder_size("tmp/agent02/services/22_test-incremental-backup-restore/data", except_files=["index.json"])
        assert size == 2097138, f"Test incremental backup failed: folder size is {size}, not 2097138 bytes"
        self.ok(nbr)

    # Tests the services sync
    def test_service_sync(self):
        nbr = "23"
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/23.jsonc BACKUP_DUMP=yes")
        print("Reloading runtime config...")
        instance.send_command("config:runtime reload")
        subprocess.run("./create_datas_stage1.sh", shell=True, check=True, cwd="tmp/agent02/services/23_test-service-sync")
        subprocess.run("./create_datas_stage2.sh", shell=True, check=True, cwd="tmp/agent02/services/23_test-service-sync")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 5 and not self.folder_size("tmp/agent01/test-storage/", except_files=["index.json"]) == 2097146:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert self.folder_size("tmp/agent01/test-storage/", except_files=["index.json"]) == 2097146, f"Test service sync failed: folder size is {self.folder_size('tmp/agent01/test-storage/', except_files=['index.json'])}, not 2097146 bytes"
        self.ok(nbr)

    # Tests the services full sync
    def test_service_full_sync(self):
        nbr = "24"
        instance = HomelabManagerInstance(nbr,["agent01", "agent02"], env="RUNTIME_CONFIG_FILE=../tests/configs/runtime/24.jsonc BACKUP_DUMP=yes")
        print("Reloading runtime config...")
        base_data_path = "tmp/agent02/services/24_test-service-full-sync/data"
        os.makedirs(base_data_path, exist_ok=True)
        if os.path.exists(os.path.join(base_data_path, "samplefile1.bin")):
            os.unlink(os.path.join(base_data_path, "samplefile1.bin"))
        if os.path.exists(os.path.join(base_data_path, "samplefile2.bin")):
            os.unlink(os.path.join(base_data_path, "samplefile2.bin"))
        instance.send_command("config:runtime reload") # config:reload does a full sync of services. This is where we will test the full sync of services.
        shutil.copyfile("tmp/agent02/services/24_test-service-full-sync/sampledatas.pdf", os.path.join(base_data_path, "samplefile1.bin"))
        shutil.copyfile("tmp/agent02/services/24_test-service-full-sync/sampledatas.pdf", os.path.join(base_data_path, "samplefile2.bin"))
        subprocess.run("./create_datas_stage1.sh", shell=True, check=True, cwd="tmp/agent02/services/24_test-service-full-sync")
        subprocess.run("./create_datas_stage2.sh", shell=True, check=True, cwd="tmp/agent02/services/24_test-service-full-sync")
        s = time.time()
        print("Waiting for test result.", end="", flush=True)
        while time.time() - s < 5 and not self.folder_size("tmp/agent01/test-storage/", except_files=["index.json"]) == 4194284:
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert self.folder_size("tmp/agent01/test-storage/", except_files=["index.json"]) == 4194284, f"Test service sync failed: folder size is {self.folder_size('tmp/agent01/test-storage/', except_files=['index.json'])}, not 4194284 bytes"
        self.ok(nbr)

    def folder_size(self, folder: str, except_files: list[str] = []) -> int:
        total_size = 0
        for dirpath, _, filenames in os.walk(folder):
            for f in filenames:
                if f in except_files:
                    continue
                fp = os.path.join(dirpath, f)
                total_size += os.path.getsize(fp)
        return total_size

    def ok(self, n):
        print(f"\033[92mTest {n} passed\033[0m")
        self.nbr_success += 1

def main():
    global server, kill_switch
    args = sys.argv[1:]
    if len(args) == 0:
        print("Disclaimer: PLEASE RUN ./test_up.sh AFTER A REBOOT AND BEFORE RUNNING THIS TEST SCRIPT!")
        print("Usage: python test_homelab_manager.py [test_numbers|all|logs|clear]")
        print("Available tests:")
        print("1: test_service_basic")
        print("2: test_generator_1")
        print("3: test_generator_2")
        print("4: test_uservar_1")
        print("5: test_uservar_2")
        print("6: test_multiple_servers_basic_01")
        print("7: test_multiple_servers_ip_01")
        print("8: test_multiple_servers_ip_with_change_01")
        print("9: test_reload_config")
        print("10: test_regen_config")
        print("11: test_runtime_config")
        print("12: test_exec_raw_restart")
        print("13: test_services_sync")
        print("14: test_var_set")
        print("15: test_emergency_proc")
        print("16: test_service_build")
        print("17: test_security_regen_cert")
        print("18: test_backup_full")
        print("19: test_incremental_backup")
        print("20: test_incremental_backup_2")
        print("21: test_full_backup_restore")
        print("22: test_incremental_backup_restore")
        print("23: test_service_sync")
        print("24: test_service_full_sync")
        print("all: run all tests")
        print("logs: print logs from the previous test run (requires the cache to be present)")
        print("logs:reload [agentX|server]: shows the logs of the specified agent or server and auto reloads the logs every 0.5s")
        print("clear: clear the test cache")
        sys.exit(1)
    map = {
        "1": "test_service_basic",
        "2": "test_generator_1",
        "3": "test_generator_2",
        "4": "test_uservar_1",
        "5": "test_uservar_2",
        "6": "test_multiple_servers_basic_01",
        "7": "test_multiple_servers_ip_01",
        "8": "test_multiple_servers_ip_with_change_01",
        "9": "test_reload_config",
        "10": "test_regen_config",
        "11": "test_runtime_config",
        "12": "test_exec_raw_restart",
        "13": "test_services_sync",
        "14": "test_var_set",
        "15": "test_emergency_proc",
        "16": "test_service_build",
        "17": "test_security_regen_cert",
        "18": "test_backup_full",
        "19": "test_incremental_backup",
        "20": "test_incremental_backup_2",
        "21": "test_full_backup_restore",
        "22": "test_incremental_backup_restore",
        "23": "test_service_sync",
        "24": "test_service_full_sync"
    }
    all_args = [str(i) for i in range(1, len(map) + 1)]
    if args[0] == "all":
        args = all_args
    if args[0] == "logs:reload":
        if len(args) < 2:
            print("Please specify the agent or server to show logs for (e.g. `python3 tests.py logs:reload agent01`)")
            sys.exit(1)
        target = args[1]
        while True:
            subprocess.run("clear", shell=True)
            if target.startswith("agent"):
                if os.path.exists(f"tmp/{target}/agent.log"):
                    with open(f"tmp/{target}/agent.log", "r") as log_file:
                        print(log_file.read())
                else:
                    print(f"No log file found for {target}")
            elif target == "server":
                if os.path.exists("test-server-logs.txt"):
                    with open("test-server-logs.txt", "r") as log_file:
                        print(log_file.read())
                else:
                    print("No log file found for server")
            else:
                print(f"Invalid target: {target}")
                sys.exit(1)
            time.sleep(0.5)
    if args[0] == "logs":
        print("Clients:")
        for f in os.listdir("tmp"):
            if f.startswith("agent"):
                print(f"  {f}:")
                if os.path.exists(f"tmp/{f}/agent.log"):
                    with open(f"tmp/{f}/agent.log", "r") as log_file:
                        print(log_file.read())
                else:
                    print("    No log file found")
                if input("[q]: Quit, [Enter]: Continue ") == "q":
                    sys.exit(0)
        print("Server:")
        if os.path.exists("test-server-logs.txt"):
            with open("test-server-logs.txt", "r") as log_file:
                print(log_file.read())
        sys.exit(0)
    if args[0] == "clear":
        shutil.rmtree("tmp", ignore_errors=True)
        if os.path.exists("test-server-logs.txt"):
            os.unlink("test-server-logs.txt")
        sys.exit(0)
    args = [arg for arg in args if arg in all_args] # Clear any invalid args
    print("Removing cache...")
    shutil.rmtree("tmp", ignore_errors=True)
    server = ResultServer.threaded_server(5001)
    os.makedirs("tmp", exist_ok=True)
    all_ok = True
    test_start_time = time.time()
    HomelabManagerInstance.LOG_FILE = open("test-logs.txt", "w", buffering=True)
    old_stdout = sys.stdout
    logger = Logger(HomelabManagerInstance.LOG_FILE)
    sys.stdout = logger
    print("Running tests...")
    print("\033[91m !! DISCLAIMER !! \033[00m")
    print("\033[91m IF YOU THINK THE TEST IS STUCK, OPEN ANOTHER SHELL AND TYPE `python3 tests.py logs` TO SEE THE AGENT + SERVER LOGS \033[00m")
    print("\033[91m ALSO IF YOU HAVE DOCKER IN ROOT MODE, YOU NEED TO RUN THIS AS ROOT, SINCE THE SOCKET AUTH IS ROOT-ONLY (FOR NOW) \033[00m")
    test = TestHomelabManager()
    for arg in args:
        test_method_name = map.get(arg)
        if test_method_name is None:
            print(f"\033[91mInvalid test number: {arg}\033[0m")
            all_ok = False
            continue
        test_method = getattr(test, test_method_name, None)
        if test_method is None:
            print(f"\033[91mTest method not found: {test_method_name}\033[0m")
            all_ok = False
            continue
        try:
            test_method()
        except AssertionError as e:
            print(f"\033[91mTest {arg} failed: {e}\033[0m")
            all_ok = False
        except Exception as e:
            print(f"\033[91mTest {arg} failed with unexpected error: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if all_ok:
        print(f"\033[92mPassed: {test.nbr_success}/{len(args)}\033[0m")
        print("\033[92mAll tests passed\033[0m")
    else:
        print(f"\033[92mPassed: {test.nbr_success}/{len(args)}\033[0m")
        print(f"\033[91mFailed: {len(args) - test.nbr_success}/{len(args)}\033[0m")
        print("\033[91mSome tests failed\033[0m")
    print(f"Ran {len(args)}/{len(all_args)} tests in {time.time() - test_start_time:.2f}s")
    logger.close()
    sys.stdout = old_stdout
    server.shutdown()
    kill_switch = True
    for thread in threading.enumerate():
        if thread is not threading.current_thread():
            thread.join(0.1)
    sys.exit(0 if all_ok else 1)

class Logger:
    def __init__(self, log_file):
        print(log_file.name)
        self.terminal = sys.stdout
        self.log = log_file

    def write(self, message):
        self.terminal.write(message)
        self.log.write(message)

    def flush(self):
        self.terminal.flush()
        self.log.flush()

    def close(self):
        self.log.close()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nInterrupted by user, shutting down...")
        if server is not None:
            server.shutdown()
        kill_switch = True
        for thread in threading.enumerate():
            if thread is not threading.current_thread():
                thread.join(0.1)

        sys.exit(1)
    except Exception as e:
        print(f"\033[91mUnexpected error: {e}\033[0m")
        print(traceback.format_exc())
        sys.exit(1)
