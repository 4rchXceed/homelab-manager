from plugins.emergency_events.listen_event import ListenEventEmergencyEvent
from plugins.emergency_events.server_offline import ServerOfflineEmergencyEvent
from plugins.emergency_events._template import EmergencyEventTemplate

EMERGENCY_EVENTS: dict[str, type[EmergencyEventTemplate]] = {
    "server_offline": ServerOfflineEmergencyEvent,
    "listen_event": ListenEventEmergencyEvent,
}
