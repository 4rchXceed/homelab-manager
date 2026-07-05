from logger import logger
from http.server import SimpleHTTPRequestHandler, HTTPServer
import time
import subprocess
import threading
from urllib.request import urlopen, Request

from helpers import get_current_context

class HttpToHttpsReverse(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.context = get_current_context()
        super().__init__(*args, **kwargs)

    def do_GET(self) -> None:
        try:
            r = urlopen(Request("http://localhost:5554" + self.path, headers={"Authorization": self.headers.get("Authorization", "")}), timeout=5)
        except Exception as e:
            if "401" in str(e):
                self.send_response(401)
                self.send_header("WWW-Authenticate", "Basic realm=\"Restricted\"")
                self.end_headers()
                self.wfile.write(b"Unauthorized")
                self.end_headers()
                return
            self.send_response(500)
            logger.error(f"Error fetching resource: {e}")
            self.end_headers()
            return
        self.send_response(r.status)
        self.end_headers()
        self.wfile.write(r.read())

class FileServer:
    def __init__(self, path: str) -> None:
        self.path = path
        self.context = get_current_context()
        self.process = None

    def proxy(self) -> None:
        logger.info("Starting file server proxy...")
        server_address = ("0.0.0.0", self.context.config_general.file_server_port)
        server = HTTPServer(server_address, HttpToHttpsReverse)
        ssl_context = self.context.app.ssl_ctx
        server.socket = ssl_context.wrap_socket(server.socket, server_side=True)
        logger.info("File server proxy started")
        server.serve_forever()

    def start(self):
        self.proxy_thread = threading.Thread(target=self.proxy)
        self.proxy_thread.start()
        self.run_thread = threading.Thread(target=self.run)
        self.run_thread.start()

    def is_started(self) -> bool:
        try:
            urlopen(
                f"http://localhost:5554", timeout=1
            )
            return True
        except Exception as e:
            if "401" in str(e):
                return True # Since it has auth, it's started
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
                "--user",
                self.context.config_general.fileserver_auth.split(":")[0],
                "--pass",
                self.context.config_general.fileserver_auth.split(":")[1],
                "--addr",
                "localhost:5554",
                self.path,
            ]
        )

    def stop(self) -> None:
        if self.process:
            self.process.terminate()
            self.process = None
