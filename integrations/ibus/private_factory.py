"""Register the fixture only in the runner's private IBus instance."""
import os
import gi
from engine import Prototype

gi.require_version('IBus', '1.0')
from gi.repository import IBus, GLib


def main():
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    if (os.environ.get('TYPOMORPH_PRIVATE_IBUS_DAEMON') != '1'
        or not runtime.startswith('/tmp/typomorph-ibus-daemon-')
        or os.environ.get('IBUS_ADDRESS') != 'unix:path=' + runtime + '/bus'
        or os.environ.get('DISPLAY') or os.environ.get('WAYLAND_DISPLAY')):
        raise SystemExit('Private runner required')
    IBus.init()
    bus = IBus.Bus.new()
    if not bus.is_connected():
        raise SystemExit('Private bus unavailable')
    factory = IBus.Factory.new(bus.get_connection())
    engines = []

    def create(factory, name):
        if name != 'typomorph-private-test':
            raise ValueError('Unsupported test engine')
        engine = Prototype(bus.get_connection(), '/org/typomorph/TestEngine' + str(len(engines)))
        engines.append(engine)
        return engine

    from test_control import ObserverFixture
    observer = ObserverFixture(engines)
    factory.connect('create-engine', create)
    component = IBus.Component.new('org.typomorph.PrivateTest', 'Isolated fixture', '0',
                                   'MIT', 'TypoMorph', '', '', '')
    component.add_engine(IBus.EngineDesc.new('typomorph-private-test', 'Private fixture',
                         'Synthetic input only', 'en', 'MIT', 'TypoMorph', '', 'us'))
    if not bus.register_component(component):
        raise SystemExit('Private registration failed')
    print('READY', flush=True)
    GLib.MainLoop().run()


if __name__ == '__main__':
    main()
