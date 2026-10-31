// Keep validation in one place for settings UI, IPC, and externally edited files.
function defaults() {
    return {version: 1, backend: 'standard', deviceId: '', standardLayout: 'ansi104', keyboards: {}, shown: true, paused: false, corner: 'bottom-right', monitor: '',
        keySize: 34, opacity: 0.78, margin: 18, topMargin: 48, clickThrough: false, positionLocked: false,
        showModifierBadges: true, highlightModifierHolds: true, suggestLayers: true, previewLayer: 0, showOutput: true};
}
function normalize(input) {
    if (!input || typeof input !== 'object' || Array.isArray(input)) throw new Error('Settings must be an object');
    if (input.version !== undefined && input.version !== 1) throw new Error('Unsupported settings version');
    var result = defaults();
    if (input.backend === undefined) result.backend = 'vial'; // Migrate existing installations.
    for (var k in result) {
        if (input[k] === undefined) continue;
        if (typeof input[k] !== typeof result[k]) throw new Error('Invalid setting: ' + k);
        result[k] = input[k];
    }
    if (!result.keyboards || Array.isArray(result.keyboards) || Object.keys(result.keyboards).length > 64) throw new Error('Invalid saved keyboards');
    var profiles = {};
    for (var id in result.keyboards) {
        var profile = result.keyboards[id];
        if (!/^[a-f0-9]{24}$/.test(id) || !profile || typeof profile !== 'object' || Array.isArray(profile) ||
            typeof profile.name !== 'string' || profile.name.length > 160 ||
            ['standard','vial'].indexOf(profile.backend) === -1 ||
            ['ansi104','ansi87','ansi60'].indexOf(profile.standardLayout) === -1 ||
            !Number.isInteger(profile.keys) || profile.keys < 1 || profile.keys > 1024 ||
            !Number.isInteger(profile.layers) || profile.layers < 1 || profile.layers > 32) throw new Error('Invalid saved keyboard');
        profiles[id] = {name:profile.name, backend:profile.backend, standardLayout:profile.standardLayout, keys:profile.keys, layers:profile.layers};
    }
    result.keyboards = profiles;
    if (['standard','vial'].indexOf(result.backend) === -1) throw new Error('Invalid keyboard mode');
    if (result.deviceId && !/^[a-f0-9]{24}$/.test(result.deviceId)) throw new Error('Invalid keyboard selection');
    if (['ansi104','ansi87','ansi60'].indexOf(result.standardLayout) === -1) throw new Error('Invalid keyboard layout');
    if (['bottom-right','bottom-left','top-right','top-left'].indexOf(result.corner) === -1) throw new Error('Invalid corner');
    var ranges = {keySize:[28,60], opacity:[0.15,1], margin:[0,200], topMargin:[0,200], previewLayer:[0,31]};
    for (var name in ranges) {
        if (!isFinite(result[name])) throw new Error('Invalid setting: ' + name);
        result[name] = Math.max(ranges[name][0], Math.min(ranges[name][1], result[name]));
    }
    result.previewLayer = Math.floor(result.previewLayer);
    return result;
}

function selectKeyboard(config, device) {
    var saved = config.keyboards[device.id];
    return {deviceId:device.id, backend:saved ? saved.backend : (device.vial ? 'vial' : 'standard'),
        standardLayout:saved ? saved.standardLayout : config.standardLayout, previewLayer:0};
}
function saveKeyboard(config, result) {
    if (result.deviceId !== config.deviceId || !result.layout || !Array.isArray(result.layout.keys)) throw new Error('Keyboard changed during setup');
    var profiles = Object.assign({}, config.keyboards);
    profiles[config.deviceId] = {name:result.name, backend:config.backend, standardLayout:config.standardLayout,
        keys:result.layout.keys.length, layers:result.layout.layers};
    // Reuse validation before persisting a probe response.
    return normalize(Object.assign({}, config, {keyboards:profiles})).keyboards;
}
