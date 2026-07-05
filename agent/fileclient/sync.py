import subprocess


def sync(address: str, path: str, auth: str, crt_file: str) -> bool:
    process = subprocess.Popen(
        ["rclone", "copy", ":http:/", "--http-url", f"https://{auth}@{address}", path],
        env={"SSL_CERT_FILE": crt_file}
    )
    process.wait()
    return process.returncode == 0
