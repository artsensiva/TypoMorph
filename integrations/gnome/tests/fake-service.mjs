import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import {BUS_NAME, OBJECT_PATH, INTERFACE_XML, LayoutService} from '../layout-bridge@typomorph.com/service.mjs';

// Synthetic state only: no Shell imports and no device or desktop access.
const manager = {inputSources: {}};
for (const id of ['us', 'ru']) {
    const source = {type: 'xkb', id, activate() { manager.currentSource = source; }};
    manager.inputSources[id] = source;
}
manager.currentSource = manager.inputSources.us;
const service = new LayoutService(manager, () => true);
const exported = Gio.DBusExportedObject.wrapJSObject(INTERFACE_XML, service);
exported.export(Gio.DBus.session, OBJECT_PATH);
const loop = new GLib.MainLoop(null, false);
const owner = Gio.bus_own_name_on_connection(
    Gio.DBus.session, BUS_NAME, Gio.BusNameOwnerFlags.NONE,
    () => print('READY'), () => loop.quit());
loop.run();
service.stop();
exported.unexport();
Gio.bus_unown_name(owner);
