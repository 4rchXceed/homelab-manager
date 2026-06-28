from helpers import get_current_context


def report_error(title: str, message: str, level: int = 0):
    context = get_current_context()
    context.app.apprise_client.notify(title=f"[{level}/3]: {title}", body=message)
