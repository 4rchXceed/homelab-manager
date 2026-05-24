import os
import subprocess
import uuid

from messaging.log import debug


def run_command(command: str, path: str) -> int:
    """
    Using docker so all dependencies are isolated
    """
    image = "debian:bookworm"
    random_id = str(uuid.uuid4())
    tmp_dir = f"/tmp/config-gen-{random_id}"
    os.mkdir(tmp_dir)
    with open(f"{tmp_dir}/commands.sh", "w") as f:
        f.write("#!/bin/bash\napt update\ncd /mnt/workdir\n" + command)
    debug(f"Running command: {command} on {path}")
    result = subprocess.run(
        [
            "docker",
            "run",
            "--rm",
            "-v",
            f"{tmp_dir}:/mnt",
            "-v",
            f"./{path}:/mnt/workdir",
            image,
            "/bin/bash",
            "/mnt/commands.sh",
        ]
    )
    return result.returncode
