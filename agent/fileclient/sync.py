import subprocess


def sync(address: str, path: str, auth: str) -> bool:
    process = subprocess.Popen(
        ["rclone", "copy", ":http:/", "--http-url", f"http://{auth}@{address}", path]
    )
    process.wait()
    return process.returncode == 0
