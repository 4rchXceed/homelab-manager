import urllib.request
import os
import json5

def postprocess_json_file(data: dict|list, fpath: str):
    if isinstance(data, dict):
        for key, value in data.items():
            if key == "_import":
                if isinstance(value, str):
                    return parse_json_file(value if value.endswith(".jsonc") or value.endswith(".json") else value + ".jsonc", os.path.dirname(fpath))
            postprocess_json_file(data[key], fpath)
    elif isinstance(data, list):
        for _, item in enumerate(data):
            postprocess_json_file(item, fpath)

def parse_json_file(file_path: str, dir: str=os.getcwd()) -> dict:
    if file_path.startswith("http://") or file_path.startswith("https://"):
        if os.environ.get("CFG_NO_WEB_IMPORT") is not None:
            raise ValueError("Web import is disabled by CFG_NO_WEB_IMPORT environment variable")
        with urllib.request.urlopen(file_path) as url:
            data = json5.loads(url.read().decode())
    else:
        if not os.path.isabs(file_path):
            file_path = os.path.join(dir, file_path)
            if not os.path.exists(file_path):
                raise FileNotFoundError(f"File not found: {file_path} in directory {dir}")
        else:
            if not os.path.exists(file_path):
                raise FileNotFoundError(f"File not found: {file_path}")
        with open(file_path, 'r', encoding="utf-8") as file:
            data = json5.load(file)
    postprocess_json_file(data, file_path)
    print(data)
    return data
