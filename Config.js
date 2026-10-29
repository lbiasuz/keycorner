// Keep validation in one place for settings UI, IPC, and externally edited files.
function defaults() {
    return {version: 1, shown: true, paused: false, corner: 'bottom-right', monitor: '',
        keySize: 34, opacity: 0.78, margin: 18, topMargin: 48, clickThrough: false,
        suggestLayers: true, previewLayer: 0, showOutput: true};
}
function normalize(input) {
    if (!input || typeof input !== 'object' || Array.isArray(input)) throw new Error('Settings must be an object');
    if (input.version !== undefined && input.version !== 1) throw new Error('Unsupported settings version');
    var result = defaults();
    for (var k in result) {
        if (input[k] === undefined) continue;
        if (typeof input[k] !== typeof result[k]) throw new Error('Invalid setting: ' + k);
        result[k] = input[k];
    }
    if (['bottom-right','bottom-left','top-right','top-left'].indexOf(result.corner) === -1) throw new Error('Invalid corner');
    var ranges = {keySize:[28,60], opacity:[0.15,1], margin:[0,200], topMargin:[0,200], previewLayer:[0,31]};
    for (var name in ranges) {
        if (!isFinite(result[name])) throw new Error('Invalid setting: ' + name);
        result[name] = Math.max(ranges[name][0], Math.min(ranges[name][1], result[name]));
    }
    result.previewLayer = Math.floor(result.previewLayer);
    return result;
}
