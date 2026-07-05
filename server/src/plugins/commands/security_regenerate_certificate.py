import shutil
from plugins.commands._template import CommandBase
from command_context import CommandContext
from helpers import get_current_context

class SecurityRegenerateCertificateCommand(CommandBase):
    NAME = "security:regen-cert"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        context = get_current_context()
        if "no-backup" not in arguments:
            cmd_context.output_print("Backing up current certificates...")
            shutil.copy("../server.crt", "../server.crt.bak")
            shutil.copy("../server.key", "../server.key.bak")
            cmd_context.output_print("Backup completed! (at: server.crt.bak, server.key.bak)")
        cmd_context.output_print("Regenerating SSL certificates...")
        context.app.generate_cert()
        cmd_context.output_print("SSL certificates regenerated successfully!")
        cmd_context.output_print("Restart the server to apply the changes.")
        return True


    @staticmethod
    def get_help() -> str:
        return """Regenerate SSL certificates. After this you will need: 1. to restart the server, 2. to re-copy the new certificate to the agents
        Usage: security:regen-cert <no-backup>
        """
