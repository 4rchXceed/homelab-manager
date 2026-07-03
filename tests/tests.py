import sys
import time
import unittest
from homelab_manager import HomelabManagerInstance
from result_srv import ResultServer, set_test, is_success

server = ResultServer.threaded_server(5001)

class TestHomelabManager():
    def test_service_basic(self):
        nbr = "1"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        instance.send_command("service:assign 1_test-service-basic agent01")
        s = time.time()
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
        assert is_success(), "Service basic test failed"
        self.ok(nbr)

    def test_generator_1(self):
        nbr = "2"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        instance.send_command("service:assign 2_test-generator-1 agent01")
        s = time.time()
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
        assert is_success(), "Service generator 1 test failed"
        self.ok(nbr)

    def test_generator_2(self):
        nbr = "3"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"])
        instance.send_command("service:assign 3_test-generator-2 agent01")
        s = time.time()
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
        assert is_success(), "Service generator 2 test failed"
        self.ok(nbr)


    def test_uservar_1(self):
        nbr = "4"
        set_test(nbr, 1)
        instance = HomelabManagerInstance(nbr,["agent01"], "USERVAR_TESTVAR=ok")
        instance.send_command("service:assign 4_test-uservar-env agent01")
        s = time.time()
        while time.time() - s < 120 and not is_success():
            time.sleep(1)
        assert is_success(), "Service generator 2 test failed"
        self.ok(nbr)

    def ok(self, n):
        print(f"\033[92mTest {n} passed\033[0m")

if __name__ == '__main__':
    tests = sys.argv[1:]
    test = TestHomelabManager()
    if len(tests) == 0:
        print("Usage: python test_homelab_manager.py [test_number|all]")
        print("Available tests:")
        print("1: test_service_basic")
        print("2: test_generator_1")
        print("3: test_generator_2")
        print("4: test_uservar_1")
        sys.exit(1)
    if len(tests) > 0 and tests[0] == "all":
        tests = ["1", "2", "3", "4"]
    if "1" in tests:
        test.test_service_basic()
    if "2" in tests:
        test.test_generator_1()
    if "3" in tests:
        test.test_generator_2()
    if "4" in tests:
        test.test_uservar_1()
