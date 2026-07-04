import threading
import sys
import socket
import shutil
import subprocess
import os
import time

class HomelabManagerInstance:
    HOMELAB_MANAGER_SERVER_PATH = "../server"
    HOMELAB_MANAGER_CLIENT_PATH = "../agent"
    COPY_AGENT_EXCEPT = [".venv", "services", "_internal.db", "agent.log"]
    SERVER_CLI_UNIX_SOCKET = "tmp/homelabmanager.sock"

    def __init__(self, test_path: str, agents: list[str], env:str=""):
        # kill all "python3 main.py" processes
        print("Killing all 'python3 main.py' processes... (use --no-kill to skip this step)")
        if not "--no-kill" in sys.argv:
            subprocess.run("pkill -f 'python3 main.py'", shell=True)
        self.env = env
        if not os.path.exists(os.path.join(self.HOMELAB_MANAGER_SERVER_PATH, "tests")):
            os.symlink(os.path.join(os.getcwd()), os.path.join(self.HOMELAB_MANAGER_SERVER_PATH, "tests"))
        print(f"Creating {len(agents)} agents... (copying code from ../agent)")
        for agent in agents:
            self.create_agent(agent)
        self.config_path = f"../tests/configs/tests/{test_path}.jsonc"
        print(f"Starting server with config {self.config_path}...")
        self.start_server()
        self.wait_start()
        print(f"Connecting to server CLI socket {self.SERVER_CLI_UNIX_SOCKET}...")
        self.connect_cli()
        for agent in agents:
            print("Registering server " + agent + " on the server...")
            self.send_command(f"server:add {agent}")
        for agent in agents:
            print(f"Starting client {agent}...")
            self.start_client(agent)
            time.sleep(1)  # Wait a bit for the agent to be fully started, maybe better method
        for agent in agents:
            print(f"Waiting for agent {agent} to be registered on the server.", end="", flush=True)
            while not self.send_command(f"exec:raw service {agent} list").strip().endswith("OK"):
                time.sleep(1)
                print(".", end="", flush=True)
            print("ok")
        print("Waiting (1s)...")
        time.sleep(1)  # Wait a bit for the agent to be fully registered, maybe better method

    def send_command(self, command: str, inputs: list[str] = []):
        self.cli.sendall(command.encode())
        response_txt = ""
        while not response_txt.strip().endswith("OK") and not response_txt.strip().endswith("ERROR"):
            response = self.cli.recv(1024)
            if response.decode().endswith("::INPUT"):
                if len(inputs) == 0:
                    raise Exception("Server requested input but no input was provided.")
                self.cli.sendall((inputs.pop(0) + "\n").encode())
            response_txt += response.decode()
        time.sleep(0.5)
        return response_txt

    def create_agent(self, agent_name: str):
        agent_path = os.path.join("tmp", agent_name)
        if os.path.exists(agent_path):
            shutil.rmtree(agent_path)
        shutil.copytree(self.HOMELAB_MANAGER_CLIENT_PATH, agent_path, ignore=shutil.ignore_patterns(*self.COPY_AGENT_EXCEPT))
        shutil.copytree("./services", os.path.join(agent_path, "services")) # Bug that makes the services sync async


    def start_server(self):
        log_file = open(
            os.path.join(self.HOMELAB_MANAGER_SERVER_PATH, "tests/test-server-logs.txt"),
            "w",
            buffering=1,
        )
        if os.path.exists(self.SERVER_CLI_UNIX_SOCKET):
            os.remove(self.SERVER_CLI_UNIX_SOCKET)

        command = (
            'docker compose kill && '
            'docker compose down -v && '
            'docker compose up -d && '
            'sleep 1 && tests/wait_healthy.sh server-db-1 10 && '
            f'docker exec server-server-1 /bin/sh -c "CONFIG_FILE={self.config_path} {self.env} ./run.sh"'
        )

        subprocess.Popen(
            command,
            shell=True,
            cwd=self.HOMELAB_MANAGER_SERVER_PATH,
            stdout=log_file,
            stderr=subprocess.STDOUT,
        )
        print("Server started.")

    def start_client(self, agent_name: str):
        command = f". ../../../agent/.venv/bin/activate && CONFIG_FOLDER=../../configs/clients/{agent_name} python3 main.py"
        state_path = os.path.join("tmp", agent_name, ".agent_state")
        if os.path.exists(state_path):
            os.unlink(state_path)

        with open(os.path.join("tmp", agent_name, "agent.log"), "w") as file:
            subprocess.Popen(
                command,
                shell=True,
                cwd=os.path.join("tmp", agent_name),
                stdout=file,
                stderr=subprocess.STDOUT,
            )
        print(f"Waiting for agent {agent_name} to start", end="", flush=True)
        t = time.time()
        while not self.agent_started(state_path) or (time.time() - t) < 10:
            print(".", end="", flush=True)
            time.sleep(1)
        print("ok")

    def agent_started(self, state_path: str):
        if os.path.exists(state_path):
            with open(state_path, "r") as file:
                state = file.read().strip()
                return state == "RUNNING"
        return False

    def wait_start(self):
        print("Waiting for the server to start", end="", flush=True)
        while not os.path.exists(self.SERVER_CLI_UNIX_SOCKET):
            time.sleep(1)
            print(".", end="", flush=True)
        print("ok")
        print("Server 100% started.")

    def connect_cli(self):
        self.cli = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.cli.connect(self.SERVER_CLI_UNIX_SOCKET)

    def stop_server(self):
        command = "docker compose down -v"
        proc = subprocess.Popen(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=self.HOMELAB_MANAGER_SERVER_PATH)
        proc.wait()
