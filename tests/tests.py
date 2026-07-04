import traceback
import threading
import os
import shutil
import sys
import time
import unittest
from homelab_manager import HomelabManagerInstance
from result_srv import ResultServer, set_test, is_success

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
    def test_service_basic(self):
        nbr = "1"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        service = "1_test-service-basic"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service basic test failed"
        th.join(0.1)
        self.ok(nbr)

    def test_generator_1(self):
        nbr = "2"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "2_test-generator-1"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 1 test failed"
        th.join(0.1)
        self.ok(nbr)

    def test_generator_2(self):
        nbr = "3"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "3_test-generator-2"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        th.join(0.1)
        self.ok(nbr)


    def test_uservar_1(self):
        nbr = "4"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"], "USERVAR_TESTVAR=ok")
        print("Starting services... (may take some time)")
        service = "4_test-uservar-env"
        th = threading.Thread(target=launch_service, args=(service, instance))
        th.start()
        s = time.time()
        print("Waiting for test result", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service uservar 1 test failed"
        th.join(0.1)
        self.ok(nbr)


    def test_uservar_2(self):
        nbr = "5"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        print("Starting services... (may take some time)")
        service = "5_test-uservar-input"
        th = threading.Thread(target=launch_service, args=(service, instance, ["ok"]))
        th.start()
        s = time.time()
        print("Waiting for test result", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service uservar 2 test failed"
        th.join(0.1)
        self.ok(nbr)

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
        print("Waiting for test result", end="", flush=True)
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        assert is_success(), "Service generator 2 test failed"
        th.join(0.1)
        self.ok(nbr)

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
        print("Waiting for test result", end="", flush=True)
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
        print("Waiting for test result", end="", flush=True)
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

    def ok(self, n):
        print(f"\033[92mTest {n} passed\033[0m")

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
        print("all: run all tests")
        print("logs: print logs from the previous test run (requires the cache to be present)")
        print("clear: clear the test cache")
        sys.exit(1)
    if args[0] == "all":
        args = ["1", "2", "3", "4", "5", "6", "7", "8"]
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
                if input("[q]: Quit, [Enter]: Continue") == "q":
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
    print("Removing cache...")
    shutil.rmtree("tmp", ignore_errors=True)
    server = ResultServer.threaded_server(5001)
    os.makedirs("tmp", exist_ok=True)
    all_ok = True
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
    if all_ok:
        print("\033[92mAll tests passed\033[0m")
    server.shutdown()
    kill_switch = True
    for thread in threading.enumerate():
        if thread is not threading.current_thread():
            thread.join(0.1)
    sys.exit(0 if all_ok else 1)

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
