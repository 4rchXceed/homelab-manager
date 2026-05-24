import json
import os


def get_config() -> str:
    cfg_env = os.getenv("CONFIG_FILE")
    if cfg_env:
        return cfg_env
    return "../conf/config.json"


def load_config():
    with open(get_config()) as f:
        return json.load(f)
