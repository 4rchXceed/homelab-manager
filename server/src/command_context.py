class CommandContext:
    def __init__(self) -> None:
        self.output_print = lambda msg: print(msg)
        self.output_input = lambda msg: input(msg)
