import json5
import json

from plugins.emergency_actions._template import EmergencyActionTemplate
from logger import logger
from plugins.emergency_actions.library import EMERGENCY_ACTIONS


class EmergencyEventTemplate:
    def init(self, actions: list[type[EmergencyActionTemplate]]) -> None:
        """
        DO NOT TOUCH THIS METHOD
        """
        self.actions = actions

    def inject(self, config: dict):
        """
        Here you inject the class into the app so you can launch events when needed.
        You can use the config for additional parameters.
        For example, you can use the event system, accessible via get_current_context().event_manager.register_event(name, callback)
        The app's context is at get_current_context() [from helpers import get_current_context]
        You can also launch a thread loop. But try to use as few ressources as possible
        When the event needs to be called just call self.fire(args).
        Args is a dict:
            key: the variable's key
            value: the variable's value
        The post-processing will, before running the action(s), replace every instances of $key with value
        Parameters:
            - config: dict: the event listener's configuration (in emergency_proc.json or other imported config)
        """
        # TODO

    def fire(self, args: dict[str, str]):
        """
        DO NOT TOUCH THIS METHOD, ONLY CALL IT WHEN YOU NEED TO FIRE THE EMERGENCY PROCEDURE
        Parameters:
            - args: dict: key: the variable's key value: the variable's value. The post-processing will, before running the action(s), replace every instances of $key with value
        """
        json_str = json5.dumps(self.actions)
        replaced_str = json_str
        args_sorted = sorted(args.items(), key=lambda x: len(x[0]), reverse=True) # From longest to shortest key, to avoid replacing shorter keys with longer ones (ex. server and server_id)
        for key, value in args_sorted:
            replaced_str = replaced_str.replace(f"${key}", value)
        logger.info(f"Firing emergency procedure! [Source: {self.__class__.__name__}]")
        replaced_obj = json5.loads(replaced_str)
        for action in replaced_obj:
            action_class = EMERGENCY_ACTIONS.get(action["type"])
            if action_class:
                action_class().call(action)



    def cancel(self, args: dict[str, str]):
        """
        Cancels the injection of the emergency procedure.
        Parameters:
            - args: dict: key: the variable's key value: the variable's value. The post-processing will, before running the action(s), replace every instances of $key with value
        """
        # TODO
