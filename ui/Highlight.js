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

// Active output modifiers, including holds produced by dual-role keys.
function modifiers(pressed, backend) {
    var codes = backend === 'standard' ? [29,42,56,125,97,54,100,126] : [224,225,226,227,228,229,230,231]
    var labels = ['Ctrl','Shift','Alt','Super','RCtrl','RShift','AltGr','RSuper']
    var active = []
    for (var i = 0; i < codes.length; ++i) {
        if (pressed.indexOf(codes[i]) >= 0) active.push({label:labels[i],bit:1 << i,usage:codes[i]})
    }
    return active
}

function holdMask(code, protocol) {
    if (code >= 224 && code <= 231) return 1 << (code - 224)
    var modtap = protocol < 6 ? (code >= 0x6000 && code <= 0x7fff) : (code >= 0x2000 && code <= 0x3fff)
    if (!modtap) return 0
    var mods = (code >> 8) & 31
    return (mods & 16) ? ((mods & 15) << 4) : (mods & 15)
}
// Modifier outputs are certain; their physical source is inferred for mod-taps.
// Do not choose among duplicate assignments or mistake # for a Shift hold.
function holds(layoutKeys, layer, liveKeys, pressed, backend, protocol) {
    var active = modifiers(pressed, backend), mask = 0, counts = {}, candidates = [], synthetic = 0
    for (var i = 0; i < active.length; ++i) mask |= active[i].bit
    for (var j = 0; j < layoutKeys.length; ++j) {
        var key = layoutKeys[j], live = liveKeys && liveKeys[j]
        var usage = live ? live.usage : key.usages[layer]
        var code = live ? live.code : (key.codes ? key.codes[layer] : 0)
        if (!live && code === 1) code = key.codes[0]
        if (backend === 'standard') {
            candidates.push({direct:true, mask:0, usage:usage})
            continue
        }
        var required = requiredModifiers(code)
        if (pressed.indexOf(usage) >= 0 && (mask & required) === required) synthetic |= required
        var assigned = holdMask(code, protocol)
        if (assigned) counts[assigned] = (counts[assigned] || 0) + 1
        candidates.push({direct:code >= 224 && code <= 231, mask:assigned, usage:usage})
    }
    return candidates.map(function(c) {
        if (backend === 'standard') return active.some(function(m) { return m.usage === c.usage })
        if (!c.mask || (mask & c.mask) !== c.mask) return false
        if (c.direct) return true
        return counts[c.mask] === 1 && !(synthetic & c.mask) && pressed.indexOf(c.usage) < 0
    })
}
