import subprocess


def sync(address: str, path: str) -> bool:
    process = subprocess.Popen(
        ["rclone", "sync", ":http:/", "--http-url", f"http://{address}", path]
    )
    process.wait()
    return process.returncode == 0
