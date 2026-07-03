import json
import os
import threading
from config.parser import parse_json_file

from logger import logger


def check_interactive_env() -> bool:
    interactive = True

    def th_input():
        nonlocal interactive
        try:
            input()
        except Exception:
            interactive = False

    thread = threading.Thread(target=th_input)
    thread.start()
    thread.join(0.1)

    return interactive


def get_config() -> str:
    cfg_env = os.getenv("CONFIG_FILE", "../conf/config.jsonc")

    if not os.path.exists(cfg_env + ".donottouch.internal"):
        if not check_interactive_env():
            logger.warning(
                "!!! IMPORTANT !!!\n"
                "YOU ARE NOT IN AN INTERACTIVE ENVIRONMENT!!\n"
                "SINCE A LOT OF CONFIGS REQUIRES INTERACTIVE VARIABLES, AND SOME OF THEM ARE REQUIRED FROM THE START OF THE PROGRAM, THE SERVER NEEDS TO:\n"
                "- EITHER USE AN INTERACTIVE ENVIRONMENT\n"
                '- ...OR (IF YOU KNOW THAT YOU DON\'T HAVE ANY "UserVar" IN YOUR CONFIG) -> SET BYPASS_TTY_INIT_CHECK=any-value\n'
                "- ...OR SET *ALL* OF THE VARIABLES IN A .env (or in env vars) (prefix: USERVAR_<UserVarIdUpperCase>) AND SET BYPASS_TTY_INIT_CHECK=any-value\n"
                ""
            )
            if not os.getenv("BYPASS_TTY_INIT_CHECK", False):
                exit(1)
    return cfg_env


def load_config() -> tuple[dict, str]:
    cfg_path = get_config()
    if not os.path.exists(cfg_path + ".donottouch.internal"):
        file = parse_json_file(cfg_path)
        return file, json.dumps(file)
    else:
        file = parse_json_file(cfg_path + ".donottouch.internal")
        return file, json.dumps(file)


def load_new_config() -> tuple[dict, str]:
    with open(
        os.getenv("CONFIG_FILE", "../conf/config.jsonc"), "r", encoding="utf-8"
    ) as f:
        text = f.read()
    return json.loads(text), text
