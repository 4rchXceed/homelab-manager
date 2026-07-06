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

def parse_time(time_str: str) -> int:
    """
    time_str: example: 12h30min
    Support:
        - y (years)
        - m (months)
        - w (weeks)
        - d (days)
        - h (hours)
        - m (minutes)
    """
    time_str = time_str.strip().lower()
    time_map = {
        "y": 365 * 24 * 60,
        "m": 30 * 24 * 60,
        "w": 7 * 24 * 60,
        "d": 24 * 60,
        "h": 60,
        "m": 1,
    }
    total = 0
    s_chars = list(time_str)
    for char, value in time_map.items():
        for i, s_char in enumerate(s_chars):
            if s_char == char:
                current = s_chars[i - 1]
                total_str = ""
                j = i
                while j >= 0 and current.isdigit():
                    total_str = current + total_str
                    j -= 1
                    current = s_chars[j]
                total += int(total_str) * value
    return int(total)
