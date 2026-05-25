import subprocess
import threading

from helpers import get_current_context


class FileServer:
    def __init__(self, path: str) -> None:
        self.path = path
        self.context = get_current_context()
        self.process = None

    def start(self):
        self.run_thread = threading.Thread(target=self.run)
        self.run_thread.start()

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
