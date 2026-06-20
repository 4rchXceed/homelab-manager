from logger import logger


class CommandContext:
    def __init__(self) -> None:
        self.output_print = lambda msg: logger.info(msg)
        self.output_input = lambda msg: input(msg)
