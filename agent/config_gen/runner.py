import os
import subprocess
import uuid

from messaging.log import debug, warning


def run_command(command: str, path: str) -> int:
    """
    Using docker so all dependencies are isolated
    """
    debug(f"Running command: {command} on {path}, mode: {command.split('::')[0]}")
    if command.startswith("FREE::"):
        warning(
            f"Running command unsandboxed! Please, do not install anything!! Running on path {path}"
        )
        res = subprocess.Popen(
            command.split("::", 1)[1],
            shell=True,
            cwd=path,
        )
        res.wait()
        return res.returncode
    elif command.startswith("SANDBOX::"):
        image = "debian:bookworm"
        random_id = str(uuid.uuid4())
        tmp_dir = f"/tmp/config-gen-{random_id}"
        os.makedirs(tmp_dir, exist_ok=True)
        with open(f"{tmp_dir}/commands.sh", "w") as f:
            f.write("#!/bin/bash\ncd /mnt/workdir\n" + command.split("::", 1)[1])
        flogs = os.open(f"{tmp_dir}/logs.txt", os.O_CREAT | os.O_WRONLY)
        result = subprocess.run(
            [
                "docker",
                "run",
                "--rm",
                "-v",
                f"{tmp_dir}/commands.sh:/commands.sh",
                "-v",
                f"./:/mnt/workdir",
                image,
                "/bin/bash",
                "/commands.sh",
            ],
            stdout=flogs,
            stderr=flogs,
            cwd=path
        )
        debug(f"Logs are available at {tmp_dir}/logs.txt")
        return result.returncode
    else:
        warning(f"Unknown running mode {command.split('::')[0]}")
        return 1
