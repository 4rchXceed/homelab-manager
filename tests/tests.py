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


    def ok(self, n):
        print(f"\033[92mTest {n} passed\033[0m")
        self.nbr_success += 1

def main():
    global server, kill_switch
    args = sys.argv[1:]
    if len(args) == 0:
        print("Disclaimer: PLEASE RUN ./test_up.sh AFTER A REBOOT AND BEFORE RUNNING THIS TEST SCRIPT!")
        print("Usage: python test_homelab_manager.py [test_number|all|logs|clear]")
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
        print("all: run all tests")
        print("logs: print logs from the previous test run (requires the cache to be present)")
        print("clear: clear the test cache")
        sys.exit(1)
    all_args = [str(i) for i in range(1, 15+1)]
    if args[0] == "all":
        args = all_args
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
    test = TestHomelabManager()
    if "1" in args:
        try:
            test.test_service_basic()
        except Exception as e:
            print(f"\033[91mTest 1 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "2" in args:
        try:
            test.test_generator_1()
        except Exception as e:
            print(f"\033[91mTest 2 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "3" in args:
        try:
            test.test_generator_2()
        except Exception as e:
            print(f"\033[91mTest 3 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "4" in args:
        try:
            test.test_uservar_1()
        except Exception as e:
            print(f"\033[91mTest 4 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "5" in args:
        try:
            test.test_uservar_2()
        except Exception as e:
            print(f"\033[91mTest 5 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "6" in args:
        try:
            test.test_multiple_servers_basic()
        except Exception as e:
            print(f"\033[91mTest 6 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "7" in args:
        try:
            test.test_multiple_servers_ip()
        except Exception as e:
            print(f"\033[91mTest 7 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "8" in args:
        try:
            test.test_multiple_servers_ip_with_change()
        except Exception as e:
            print(f"\033[91mTest 8 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "9" in args:
        try:
            test.test_reload_config()
        except Exception as e:
            print(f"\033[91mTest 9 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "10" in args:
        try:
            test.test_regen_config()
        except Exception as e:
            print(f"\033[91mTest 10 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "11" in args:
        try:
            test.test_runtime_config()
        except Exception as e:
            print(f"\033[91mTest 11 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "12" in args:
        try:
            test.test_exec_raw_restart()
        except Exception as e:
            print(f"\033[91mTest 12 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "13" in args:
        try:
            test.test_services_sync()
        except Exception as e:
            print(f"\033[91mTest 13 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "14" in args:
        try:
            test.test_var_set()
        except Exception as e:
            print(f"\033[91mTest 14 failed: {e}\033[0m")
            print(traceback.format_exc())
            all_ok = False
    if "15" in args:
        try:
            test.test_emergency_proc()
        except Exception as e:
            print(f"\033[91mTest 15 failed: {e}\033[0m")
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
