// Match output usages to the most specific active modifier assignment.
// HID output alone cannot distinguish physically identical assignments.
function requiredModifiers(code) {
    if (!(code >= 0x0100 && code <= 0x1fff)) return 0
    var mods = (code >> 8) & 31
    return (mods & 16) ? ((mods & 15) << 4) : (mods & 15)
}
function weight(bits) {
    var count = 0
    while (bits) { count += bits & 1; bits >>>= 1 }
    return count
}
function keys(layoutKeys, layer, liveKeys, pressed, backend) {
    var held = {}, modifiers = 0, candidates = [], best = {}
    for (var i = 0; i < pressed.length; ++i) {
        held[pressed[i]] = true
        if (pressed[i] >= 224 && pressed[i] <= 231) modifiers |= 1 << (pressed[i] - 224)
    }
    for (var j = 0; j < layoutKeys.length; ++j) {
        var key = layoutKeys[j], live = liveKeys && liveKeys[j]
        var usage = live ? live.usage : key.usages[layer]
        var code = live ? live.code : (key.codes ? key.codes[layer] : 0)
        // Manual previews use the same base-layer fallthrough as the layout model.
        if (!live && code === 1) code = key.codes[0]
        var required = backend === 'vial' ? requiredModifiers(code) : 0
        var match = usage > 0 && !!held[usage] && (modifiers & required) === required
        var score = weight(required)
        candidates.push({usage:usage, match:match, score:score})
        if (match && (best[usage] === undefined || score > best[usage])) best[usage] = score
    }
    return candidates.map(function(c) { return c.match && c.score === best[c.usage] })
}
