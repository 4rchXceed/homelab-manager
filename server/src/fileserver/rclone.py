import time
import subprocess
import threading
from urllib.request import urlopen

from helpers import get_current_context


class FileServer:
    def __init__(self, path: str) -> None:
        self.path = path
        self.context = get_current_context()
        self.process = None

    def start(self):
        self.run_thread = threading.Thread(target=self.run)
        self.run_thread.start()

    def is_started(self) -> bool:
        try:
            urlopen(
                "http://localhost:" + str(self.context.config_general.file_server_port), timeout=1
            )
            return True
        except Exception:
            return False

    def wait_until_started(self, timeout: float = 10.0) -> None:
        while not self.is_started() and timeout > 0:
            time.sleep(0.1)
            timeout -= 0.1

    def run(self) -> None:
        self.process = subprocess.Popen(
            [
                "rclone",
                "serve",
                "http",
                "--addr",
                "0.0.0.0:" + str(self.context.config_general.file_server_port),
                self.path,
            ]
        )

    def stop(self) -> None:
        if self.process:
            self.process.terminate()
            self.process = None
