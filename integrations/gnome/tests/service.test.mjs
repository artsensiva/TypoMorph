import test from 'node:test';
import assert from 'node:assert/strict';
import {LayoutService} from '../layout-bridge@typomorph.com/service.mjs';

function fixture() {
    let allowed = true;
    const activated = [];
    const manager = {inputSources: {}};
    for (const id of ['us', 'ru']) {
        const source = {type: 'xkb', id, activate(interactive) {
            activated.push([id, interactive]);
            manager.currentSource = source;
        }};
        manager.inputSources[id] = source;
    }
    manager.currentSource = manager.inputSources.us;
    return {manager, activated, service: new LayoutService(manager, () => allowed),
        lock() { allowed = false; }};
}

test('reports actual source and confirms its activation', () => {
    const {service, activated} = fixture();
    assert.deepEqual(service.GetState(), [1, true, 'xkb', 'us', ['us', 'ru']]);
    assert.deepEqual(service.Switch('us', 'ru'), [true, 'xkb', 'ru']);
    assert.deepEqual(activated, [['ru', false]]);
});

test('rejects a stale source without activation', () => {
    const {service, manager, activated} = fixture();
    manager.currentSource = manager.inputSources.ru;
    assert.deepEqual(service.Switch('us', 'ru'), [false, '', '']);
    assert.equal(activated.length, 0);
});

test('lock and disabled service block state and mutation', () => {
    for (const action of ['lock', 'stop']) {
        const f = fixture();
        if (action === 'lock') f.lock(); else f.service.stop();
        assert.deepEqual(f.service.GetState(), [1, false, '', '', []]);
        assert.deepEqual(f.service.Switch('us', 'ru'), [false, '', '']);
        assert.equal(f.activated.length, 0);
    }
});

test('IME and unknown current source are unavailable', () => {
    for (const source of [null, {type: 'ibus', id: 'test-ime'}, {type: 'xkb', id: 'us'}]) {
        const f = fixture();
        f.manager.currentSource = source;
        assert.equal(f.service.GetState()[1], false);
        assert.equal(f.service.Switch('us', 'ru')[0], false);
        assert.equal(f.activated.length, 0);
    }
});

test('unknown or malformed target never executes', () => {
    const f = fixture();
    for (const target of ['de', '', "ru';throw 1", 'a'.repeat(129)]) {
        assert.equal(f.service.Switch('us', target)[0], false);
    }
    assert.equal(f.activated.length, 0);
});

test('no-op and throwing activations never claim success', () => {
    for (const activate of [() => {}, () => { throw new Error('synthetic'); }]) {
        const f = fixture();
        f.manager.inputSources.ru.activate = activate;
        assert.equal(f.service.Switch('us', 'ru')[0], false);
    }
});

test('lock during activation invalidates confirmation', () => {
    const f = fixture();
    f.manager.inputSources.ru.activate = () => {
        f.manager.currentSource = f.manager.inputSources.ru;
        f.lock();
    };
    assert.equal(f.service.Switch('us', 'ru')[0], false);
});

test('duplicate IDs and removed source are rejected', () => {
    const f = fixture();
    f.manager.inputSources.duplicate = {type: 'xkb', id: 'us'};
    assert.equal(f.service.GetState()[1], false);
    assert.equal(f.service.Switch('us', 'ru')[0], false);
    assert.equal(f.activated.length, 0);
});
