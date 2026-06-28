from helpers import get_current_context
from logger import logger
from plugins.emergency_events._template import EmergencyEventTemplate


class ListenEventEmergencyEvent(EmergencyEventTemplate):
    def inject(self, config: dict):
        if not config.get("event_name"):
            logger.warning("event_name is required")
            return
        self.event_name = config.get("event_name","")
        context = get_current_context()
        context.event_manager.register_event(self.event_name, self.prefire)

    def prefire(self, *args, **kwargs):
        args = {}
        for i, arg in enumerate(args):
            args[str(i)] = arg
        for key, value in kwargs.items():
            args[key] = value
        self.fire(args)

    def cancel(self, args: dict[str, str]):
        if not self.event_name:
            return
        context = get_current_context()
        context.event_manager.unregister_event(self.event_name, self.prefire)
