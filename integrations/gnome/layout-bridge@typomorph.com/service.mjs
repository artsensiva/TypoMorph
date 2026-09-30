export const BUS_NAME = 'org.typomorph.Layout1';
export const OBJECT_PATH = '/org/typomorph/Layout1';
export const INTERFACE_XML = `<node>
<interface name="org.typomorph.Layout1">
  <method name="GetState">
    <arg type="u" direction="out" name="version"/>
    <arg type="b" direction="out" name="ready"/>
    <arg type="s" direction="out" name="sourceType"/>
    <arg type="s" direction="out" name="sourceId"/>
    <arg type="as" direction="out" name="layouts"/>
  </method>
  <method name="Switch">
    <arg type="s" direction="in" name="expectedSource"/>
    <arg type="s" direction="in" name="target"/>
    <arg type="b" direction="out" name="confirmed"/>
    <arg type="s" direction="out" name="sourceType"/>
    <arg type="s" direction="out" name="sourceId"/>
  </method>
</interface>
</node>`;

const validId = id => typeof id === 'string' && /^[a-zA-Z0-9_+\-]{1,128}$/.test(id);

// The service has no input events, window text, files, network, or eval access.
// Dependencies are injected so the actual policy can be tested outside Shell.
export class LayoutService {
    constructor(manager, available) {
        this.manager = manager;
        this.available = available;
        this.enabled = true;
    }

    stop() {
        this.enabled = false;
    }

    GetState() {
        try {
            if (!this.enabled || !this.available())
                return [1, false, '', '', []];
            const source = this.manager.currentSource;
            const sources = Object.values(this.manager.inputSources);
            if (sources.length > 128)
                return [1, false, '', '', []];
            const layouts = sources.filter(s => s.type === 'xkb' && validId(s.id)).map(s => s.id);
            if (!source || source.type !== 'xkb' || !validId(source.id) ||
                !sources.includes(source) || new Set(layouts).size !== layouts.length)
                return [1, false, '', '', []];
            return [1, true, 'xkb', source.id, layouts];
        } catch {
            return [1, false, '', '', []];
        }
    }

    Switch(expectedSource, targetId) {
        const [, ready, , current, layouts] = this.GetState();
        if (!ready || !validId(expectedSource) || !validId(targetId) ||
            current !== expectedSource || !layouts.includes(targetId))
            return [false, '', ''];
        try {
            const target = Object.values(this.manager.inputSources)
                .find(s => s.type === 'xkb' && s.id === targetId);
            if (!target)
                return [false, '', ''];
            target.activate(false);
            const [, stillReady, type, id] = this.GetState();
            if (!stillReady || this.manager.currentSource !== target || id !== targetId)
                return [false, '', ''];
            return [true, type, id];
        } catch {
            return [false, '', ''];
        }
    }
}
