from context import HLMContext

current_context = None


def get_current_context() -> HLMContext:
    global current_context
    if current_context is None:
        raise RuntimeError(
            "Trying to get current context with fail, but it doesn't exists. Hint: Something at the start of the program is crashing, probably. (DB connection?)"
        )
    return current_context


def get_current_context_nofail() -> HLMContext | None:
    global current_context
    return current_context


def set_current_context(context: HLMContext) -> None:
    global current_context
    current_context = context
