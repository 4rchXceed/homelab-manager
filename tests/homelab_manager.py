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

    def __init__(self, test_nbr: str, servers: list[str], env:str=""):
        # kill all "python3 main.py" processes
        subprocess.run("pkill -f 'python3 main.py'", shell=True)
        self.env = env
        if not os.path.exists(os.path.join(self.HOMELAB_MANAGER_SERVER_PATH, "tests")):
            os.symlink(os.path.join(os.getcwd()), os.path.join(self.HOMELAB_MANAGER_SERVER_PATH, "tests"))
        for server in servers:
            self.create_agent(server)
        self.config_path = f"../tests/configs/tests/{test_nbr}.jsonc"
        self.start_server()
        self.wait_start()
        self.connect_cli()
        for server in servers:
            self.send_command(f"server:add {server}")
        for server in servers:
            self.start_client(server)
        time.sleep(4) # TODO: Find better method
        for server in servers:
            while not self.send_command(f"exec:raw service {server} list").strip().endswith("OK"):
                time.sleep(1)

    def send_command(self, command: str):
        self.cli.sendall(command.encode())
        print(f"Sent command: {command}")
        response_txt = ""
        while not response_txt.strip().endswith("OK") and not response_txt.strip().endswith("ERROR"):
            response = self.cli.recv(1024)
            response_txt += response.decode()
            print(f"Received response: {response.decode()}")
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

        proc = subprocess.Popen(
            command,
            shell=True,
            cwd=self.HOMELAB_MANAGER_SERVER_PATH,
            stdout=log_file,
            stderr=subprocess.STDOUT,
        )
        print("Server started.")

    def start_client(self, agent_name: str):
        print(f"Starting client {agent_name}...")
        command = f". ../../../agent/.venv/bin/activate && CONFIG_FOLDER=../../configs/clients/{agent_name} python3 main.py"

        with open(os.path.join("tmp", agent_name, "agent.log"), "w") as file:
            process = subprocess.Popen(
                command,
                shell=True,
                cwd=os.path.join("tmp", agent_name),
                stdout=file,
                stderr=subprocess.STDOUT,
            )

        print(f"Client {agent_name} started.")

    def wait_start(self):
        while not os.path.exists(self.SERVER_CLI_UNIX_SOCKET):
            print("Waiting for the server to start...")
            time.sleep(1)
        print("Server 100% started.")

    def connect_cli(self):
        self.cli = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.cli.connect(self.SERVER_CLI_UNIX_SOCKET)

    def stop_server(self):
        command = "docker compose down -v"
        proc = subprocess.Popen(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=self.HOMELAB_MANAGER_SERVER_PATH)
        proc.wait()
