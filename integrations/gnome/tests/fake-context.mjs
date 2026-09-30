import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
let mode = 0;
let textQueries = 0;
let roleQueries = 0;
let stateQueries = 0;
const connection = Gio.DBus.session;
const app = 'org.a11y.atspi.Registry';
const root = '/org/a11y/atspi/accessible/root';
const accessibleXml = `<node><interface name="org.a11y.atspi.Accessible">
<method name="GetChildren"><arg type="a(so)" direction="out"/></method>
<method name="GetApplication"><arg type="(so)" direction="out"/></method>
<method name="GetRole"><arg type="u" direction="out"/></method>
<method name="GetState"><arg type="au" direction="out"/></method>
<method name="GetInterfaces"><arg type="as" direction="out"/></method>
</interface></node>`;
const textXml = `<node><interface name="org.a11y.atspi.Text">
<property name="CaretOffset" type="i" access="read"/>
<property name="CharacterCount" type="i" access="read"/>
<method name="GetNSelections"><arg type="i" direction="out"/></method>
</interface></node>`;
const exports = [];
function expose(xml, object, path) {
    const service = Gio.DBusExportedObject.wrapJSObject(xml, object);
    service.export(connection, path);
    exports.push(service);
}
expose(`<node><interface name="org.a11y.Bus"><method name="GetAddress">
<arg type="s" direction="out"/></method></interface></node>`,
    {GetAddress() {return GLib.getenv('DBUS_SESSION_BUS_ADDRESS');}}, '/org/a11y/bus');
expose(`<node><interface name="org.typomorph.ContextFixture">
<method name="SetMode"><arg type="u" direction="in"/></method>
<method name="TextQueries"><arg type="u" direction="out"/></method>
</interface></node>`, {SetMode(value) {mode = value; textQueries = 0; roleQueries = 0; stateQueries = 0;},
    TextQueries() {return textQueries;}}, '/fixture');

for (const [path, role, child] of [
    [root, 14, '/app'], ['/app', 75, '/window'], ['/window', 23, '/entry'], ['/entry', 61, null],
]) {
    expose(accessibleXml, {
        GetChildren() {
            if (!child) return [];
            const children = [[app, '/org/a11y/atspi/null'], [app, child],
                [app, '/org/a11y/atspi/null']];
            if (mode === 7 && path === '/window') children.push([app, '/missing']);
            return children;
        },
        GetApplication() {return [app, mode === 15 ? "/missing" : "/app"];},
        GetRole() {
            if (path !== '/entry') return role;
            if (mode === 6 && ++roleQueries >= 2) {textQueries = 0; return 40;}
            return mode === 1 || mode === 14 ? 40 : role;
        },
        GetState() {
            if (path === '/window') return [mode === 4 ? 0 : 2, 0];
            if (path !== '/entry') return [0, 0];
            const bits = [7, 8, 12, 24, 25, 30].reduce((state, bit) => state | (1 << bit), 0);
            let low = mode === 2 ? bits & ~(1 << 12) : bits;
            if (mode >= 8) low &= ~(1 << 8);
            if (mode === 9 || (mode === 16 && ++stateQueries >= 2)) low &= ~(1 << 24);
            return [low, mode === 10 ? 1 << (43 - 32) : 0];
        },
        GetInterfaces() {return ['org.a11y.atspi.Accessible', 'org.a11y.atspi.Text'];},
    }, path);
}
expose(`<node><interface name="org.a11y.atspi.Application">
<property name="ToolkitName" type="s" access="read"/>
<property name="Version" type="s" access="read"/>
</interface></node>`, {
    get ToolkitName() {return mode === 11 ? 'other' : 'GTK';},
    get Version() {return mode === 12 ? '3.24.0' : mode === 13 ? 'invalid' : '4.22.4';},
}, '/app');
expose(textXml, {
    get CaretOffset() {textQueries++; return mode === 5 ? -1 : 3;},
    get CharacterCount() {textQueries++; return 10;},
    GetNSelections() {textQueries++; return mode === 3 ? 1 : 0;},
}, '/entry');

let owned = 0;
const loop = new GLib.MainLoop(null, false);
const owners = ['org.a11y.Bus', app].map(name =>
    Gio.bus_own_name_on_connection(connection, name, Gio.BusNameOwnerFlags.NONE,
        () => {if (++owned === 2) print('READY');}, () => loop.quit()));
loop.run();
for (const service of exports) service.unexport();
for (const owner of owners) Gio.bus_unown_name(owner);
