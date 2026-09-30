import Gio from 'gi://Gio';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {getInputSourceManager} from 'resource:///org/gnome/shell/ui/status/keyboard.js';
import {BUS_NAME, OBJECT_PATH, INTERFACE_XML, LayoutService} from './service.mjs';

export default class TypoMorphLayoutBridge extends Extension {
    enable() {
        this.disable();
        const manager = getInputSourceManager();
        this._service = new LayoutService(manager, () =>
            Main.sessionMode.hasWindows &&
            !Main.sessionMode.isLocked &&
            !Main.sessionMode.isGreeter &&
            !manager.keyboardManager.isLocked());
        const service = this._service;
        try {
            this._exported = Gio.DBusExportedObject.wrapJSObject(INTERFACE_XML, this._service);
            this._exported.export(Gio.DBus.session, OBJECT_PATH);
            this._owner = Gio.bus_own_name_on_connection(
                Gio.DBus.session, BUS_NAME, Gio.BusNameOwnerFlags.NONE,
                null, () => service.stop());
        } catch (error) {
            this.disable();
            throw error;
        }
    }

    disable() {
        this._service?.stop();
        if (this._owner) {
            Gio.bus_unown_name(this._owner);
            this._owner = 0;
        }
        this._exported?.unexport();
        this._exported = null;
        this._service = null;
    }
}
