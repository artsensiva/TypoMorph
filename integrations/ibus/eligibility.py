"""Content-free focus-generation gate. A real trusted field observer is still required."""
from secrets import randbits


class Eligibility:
    def __init__(self):
        self.context = None
        self.field = None
        self.generation = randbits(48)
        self.permitted = False
        self.pending = False

    def invalidate(self):
        self.generation += 1
        self.field = None
        self.permitted = False
        self.pending = False

    def focus(self, context):
        self.invalidate()
        self.context = context

    def begin(self, context, field):
        if not context or context != self.context or not field or len(field) > 256:
            return 0
        self.invalidate()
        self.field = field
        self.pending = True
        return self.generation

    def confirm(self, context, field, generation, safe):
        if not (self.pending and self.context == context and self.field == field
                and self.generation == generation):
            return False
        self.pending = False
        self.permitted = bool(safe)
        return self.permitted
