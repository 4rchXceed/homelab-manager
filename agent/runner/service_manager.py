from messaging.log import debug
import threading
import json
import os
import subprocess
from watchfiles import watch

# Utility classes


class ServiceStatus:
    def __init__(
        self, name: str, running: bool, healthy: None | bool, line_str: str
    ) -> None:
        self.name = name
        self.running = running
        self.healthy: None | bool = healthy
        self.line_str = line_str


class ServiceManager:
    MAX_B64_SIZE = 1024*1024 # How many max bytes of base64 data to send in a single message, so it doesn't destroy the performance during the encoding and sending process. 1MB is a good limit for now.
    def __init__(self, services_folder, add_file_to_queue, send_message) -> None:
        self.services_folder = services_folder
        self.add_file_to_queue = add_file_to_queue
        self.send_message = send_message

    def watch_files(self, service_name, datas: list[str]) -> None:
        service_path = os.path.join(self.services_folder, service_name)
        service_data_paths = [os.path.join(service_path, data.removeprefix("./")) for data in datas]
        for data_path in service_data_paths:
            if not os.path.exists(data_path):
                os.makedirs(data_path, exist_ok=True)
            debug(f"Watching path: {data_path}")
        for changes in watch(*service_data_paths, stop_event=threading.Event()):
            for change in changes:
                file_path = change[1]
                change_type = change[0]

                clean_file_name = os.path.abspath(file_path).removeprefix(os.path.abspath(service_path)).removeprefix(os.path.sep)
                debug(f"File change detected: {file_path} (type: {change_type})")
                if change_type in [1,2]:
                    self.add_file_to_queue(file_path, clean_file_name, service_name)
                else:
                    self.send_message({
                        "type": "relay_file_deletion",
                        "service": service_name,
                        "file_name": clean_file_name
                    })

    def delete_file(self, service: str, file_name: str, path: str) -> None:
        service_path = os.path.join(path, service)
        file_path = os.path.join(service_path, "sync", file_name)
        if os.path.exists(file_path):
            os.remove(file_path)

    def start_watchfiles(self, service_name:str, datas:list[str]) -> threading.Thread:
        th = threading.Thread(target=self.watch_files, args=(service_name,datas))
        th.start()
        return th

    def is_running(self, service_name) -> bool:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            result = subprocess.run(
                ["docker", "compose", "ps"], capture_output=True, text=True, cwd=service_name
            )
            if result.returncode == 0:
                if result.stdout.strip() != "":
                    return True
            return False
        return False

    def build(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            result = subprocess.run(
                ["docker", "compose", "build"], capture_output=True, text=True, cwd=service_name
            )
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def start(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            result = subprocess.run(
                ["docker", "compose", "up", "-d"], capture_output=True, text=True, cwd=service_name
            )
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def stop(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            result = subprocess.run(
                ["docker", "compose", "down"], capture_output=True, text=True, cwd=service_name
            )
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def restart(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            result = subprocess.run(
                ["docker", "compose", "restart"], capture_output=True, text=True, cwd=service_name
            )
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def list_services(self) -> list[ServiceStatus]:
        services = []
        if not os.path.exists(self.services_folder):
            return services
        for service_name in os.listdir(self.services_folder):
            service_path = os.path.join(self.services_folder, service_name)
            if os.path.exists(service_path) and os.path.isdir(service_path):
                result = subprocess.run(
                    ["docker", "compose", "ps", "--format", "json"],
                    capture_output=True,
                    text=True,
                    cwd=service_path
                )
                if result.returncode == 0:
                    if result.stdout.strip() != "":
                        try:
                            datas = json.loads(result.stdout)
                            health = datas.get("Health", "")
                            services.append(
                                ServiceStatus(
                                    service_name,
                                    True,
                                    health == "healthy"
                                    if health != "starting"
                                    else None,
                                    datas["Status"],
                                )
                            )
                        except Exception:
                            services.append(
                                ServiceStatus(service_name, False, None, result.stdout)
                            )

                    else:
                        services.append(
                            ServiceStatus(service_name, False, None, result.stdout)
                        )
                else:
                    services.append(
                        ServiceStatus(service_name, False, None, result.stderr)
                    )
        return services
