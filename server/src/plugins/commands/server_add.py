import uuid

from command_context import CommandContext
from database.models import Server
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ServerAddCommand(CommandBase):
    NAME = "server:add"

    @staticmethod
    def execute(arguments, cmd_context: CommandContext) -> bool:
        context = get_current_context()
        server_name = arguments[0] if len(arguments) > 0 else None
        if not server_name:
            cmd_context.output_print("Server name is required")
            return False
        cfg_server = None
        i = 0
        while i < len(context.config_servers.servers) and not cfg_server:
            if context.config_servers.servers[i]["id"] == server_name:
                cfg_server = context.config_servers.servers[i]
            i += 1
        if not cfg_server:
            cmd_context.output_print("Server not found in the config")
            return False
        id_str = cfg_server.get("id")
        name = cfg_server.get("name")
        description = cfg_server.get("description")
        ip = cfg_server.get("ip")
        if not id_str or not name or not ip:
            cmd_context.output_print(
                "Server id, name, and ip are required in the config!!!"
            )
            return False

        db_server = (
            context.database.session.query(Server).filter_by(id_str=id_str).first()
        )
        if db_server:
            if len(arguments) > 1 and arguments[1] == "can_update":
                cmd_context.output_print(
                    "Server already exists in the database, updating due to can_update flag"
                )
            else:
                cmd_context.output_print(
                    "Server already exists in the database. Failed to add."
                )
                return False
        else:
            db_server = Server(id_str=id_str, name=name, description=description, ip=ip)
            context.database.session.add(db_server)
            context.database.session.commit()
            cmd_context.output_print("Server added to the database")
        api_key = str(uuid.uuid4())
        cmd_context.output_print(f"New API key: {api_key}")
        db_server.api_key = api_key
        context.database.session.commit()
        return True

    @staticmethod
    def get_help() -> str:
        return "Add a new server THAT IS ALREADY IN THE CONFIG BUT NOT IN THE DATABASE (generates the api key) (args: server_name [can_update]). can_update defines if the command can update the server's api key, or if it should fail if the server already exists in the database"
