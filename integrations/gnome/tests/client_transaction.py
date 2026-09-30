"""Test-owned GTK transaction experiment, not an adapter for arbitrary editors."""
from dataclasses import dataclass
import time
import gi

gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, GLib


@dataclass
class Request:
    epoch: int
    original: str
    caret: int
    deadline: float


class FieldOwner:
    """One short synthetic word per field; all operations on the GTK UI thread.

    The test editor owns its buffer and callbacks. Buffer replacement is used
    instead of delete-then-insert. External editor callbacks, undo and shared
    buffers are deliberately unsupported, rather than claimed atomic.
    """
    def __init__(self, widget, ordinary=False):
        self.widget = widget
        self.ordinary = ordinary
        self.alive = True
        self.connected = True
        self.composing = False
        self.applying = False
        self.epoch = 0
        self.sequence = 0
        self.pending = {}
        self.snapshots = 0
        self.requests = 0
        self.applied = 0
        self.context = next(c.get_im_context() for c in
            (widget.observe_controllers().get_item(i)
             for i in range(widget.observe_controllers().get_n_items()))
            if isinstance(c, Gtk.EventControllerKey) and c.get_im_context())
        widget.connect('changed', self.invalidate)
        widget.get_root().connect('notify::is-active', self.invalidate)
        for prop in ('has-focus', 'cursor-position', 'selection-bound',
                     'visibility', 'input-purpose', 'buffer'):
            widget.connect('notify::'+prop, self.invalidate)
        self.context.connect('preedit-start', self._composition_start)
        self.context.connect('preedit-end', self._composition_end)

    def invalidate(self, *unused):
        self.epoch += 1
        self.pending.clear()

    def _composition_start(self, *unused):
        self.composing = True
        self.invalidate()

    def _composition_end(self, *unused):
        self.composing = False
        self.invalidate()

    def set_ordinary(self, value):
        self.ordinary = value
        self.invalidate()

    def reset(self):
        self.invalidate()
        self.context.reset()

    def disconnect(self):
        self.connected = False
        self.invalidate()

    def reconnect(self):
        self.connected = True
        self.invalidate()

    def dispose(self):
        # The owner must do this before removing/destroying its widget.
        self.alive = False
        self.invalidate()

    def eligible(self):
        w = self.widget
        return (self.alive and self.connected and not self.applying
                and self.ordinary and not self.composing and w.has_focus()
                and isinstance(w.get_root(), Gtk.Window) and w.get_root().is_active()
                and w.get_visibility()
                and w.get_input_purpose() == Gtk.InputPurpose.FREE_FORM
                and not w.get_selection_bounds())

    def request(self, timeout_ms=1000):
        # Safety metadata is checked before any text read, including length.
        if not self.eligible():
            return None
        if not 0 < self.widget.get_buffer().get_length() <= 32:
            return None
        self.snapshots += 1
        original = self.widget.get_text()
        caret = self.widget.get_position()
        if original != 'ghbdtn' or caret != len(original):
            return None
        self.sequence += 1
        self.requests += 1
        self.pending.clear()  # Only one bounded outstanding snapshot.
        ticket = object()  # Unique local identity, never reusable across owners.
        self.pending[ticket] = Request(self.epoch, original, caret, time.monotonic() + timeout_ms / 1000)
        def expire():
            self.cancel(ticket)
            return False
        GLib.timeout_add(timeout_ms, expire)
        return ticket

    def cancel(self, ticket):
        self.pending.pop(ticket, None)

    def complete(self, ticket):
        if not self.eligible():
            self.cancel(ticket)
            return False
        request = self.pending.pop(ticket, None)
        if (request is None or request.epoch != self.epoch
                or time.monotonic() >= request.deadline):
            return False
        w = self.widget
        if (w.get_buffer().get_length() != len(request.original)
                or w.get_position() != request.caret
                or w.get_text() != request.original):
            return False
        # Prepare the entire new buffer before exposing any mutation. No main
        # loop iteration, await, or delete/insert on the visible buffer here.
        replacement = Gtk.EntryBuffer.new('привет', -1)
        self.applying = True
        self.invalidate()
        try:
            w.set_buffer(replacement)
            w.set_position(6)
            # An arbitrary external callback would break this owned-editor
            # contract. Detect it; do not replay input or hide partial failure.
            if w.get_buffer() != replacement or w.get_text() != 'привет':
                raise RuntimeError('Uncontrolled mutation during owned edit')
            self.applied += 1
            return True
        finally:
            self.applying = False
