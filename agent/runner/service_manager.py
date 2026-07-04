import json
import os
import subprocess

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
    def __init__(self, services_folder) -> None:
        self.services_folder = services_folder

    def is_running(self, service_name) -> bool:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            pwd = os.getcwd()
            os.chdir(service_name)
            result = subprocess.run(
                ["docker", "compose", "ps"], capture_output=True, text=True
            )
            os.chdir(pwd)
            if result.returncode == 0:
                if result.stdout.strip() != "":
                    return True
            return False
        return False

    def build(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            pwd = os.getcwd()
            os.chdir(service_name)
            result = subprocess.run(
                ["docker", "compose", "build"], capture_output=True, text=True
            )
            os.chdir(pwd)
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def start(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            pwd = os.getcwd()
            os.chdir(service_name)
            result = subprocess.run(
                ["docker", "compose", "up", "-d"], capture_output=True, text=True
            )
            os.chdir(pwd)
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def stop(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            pwd = os.getcwd()
            os.chdir(service_name)
            result = subprocess.run(
                ["docker", "compose", "down"], capture_output=True, text=True
            )
            os.chdir(pwd)
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def restart(self, service_name) -> tuple[bool, str]:
        service_name = os.path.join(self.services_folder, service_name)
        if os.path.exists(service_name) and os.path.isdir(service_name):
            pwd = os.getcwd()
            os.chdir(service_name)
            result = subprocess.run(
                ["docker", "compose", "restart"], capture_output=True, text=True
            )
            os.chdir(pwd)
            if result.returncode == 0:
                return False, result.stdout
            return True, result.stderr
        return True, f"Service {service_name} not found"

    def list_services(self) -> list[ServiceStatus]:
        try:
            services = []
            for service_name in os.listdir(self.services_folder):
                service_path = os.path.join(self.services_folder, service_name)
                if os.path.exists(service_path) and os.path.isdir(service_path):
                    pwd = os.getcwd()
                    os.chdir(service_path)
                    print(f"Listing services in {service_path}")
                    result = subprocess.run(
                        ["docker", "compose", "ps", "--format", "json"],
                        capture_output=True,
                        text=True,
                    )
                    os.chdir(pwd)
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
        except Exception as e:
            print(f"Error occurred while listing services: {e}")
        return []
