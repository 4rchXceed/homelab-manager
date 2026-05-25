from typing import Callable


class EventManager:
    def __init__(self) -> None:
        self.events: dict[str, list[Callable]] = {}

    def register_event(self, event_name: str, callback: Callable) -> None:
        if event_name not in self.events:
            self.events[event_name] = []
        self.events[event_name].append(callback)

    def trigger_event(self, event_name: str, *args, **kwargs) -> None:
        if event_name in self.events:
            for callback in self.events[event_name]:
                callback(*args, **kwargs)

    def unregister_event(self, event_name: str, callback: Callable) -> None:
        if event_name in self.events:
            self.events[event_name].remove(callback)
