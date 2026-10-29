// Convert Qt key events to the XKB names accepted by Hyprland.
function combination(key, modifiers) {
    if ([Qt.Key_Shift, Qt.Key_Control, Qt.Key_Alt, Qt.Key_Meta, Qt.Key_AltGr].indexOf(key) >= 0) return null;
    var mods = [];
    if (modifiers & Qt.MetaModifier) mods.push('SUPER');
    if (modifiers & Qt.ControlModifier) mods.push('CTRL');
    if (modifiers & Qt.AltModifier) mods.push('ALT');
    if (modifiers & Qt.ShiftModifier) mods.push('SHIFT');
    if (!mods.length || (modifiers & Qt.GroupSwitchModifier)) return '';
    var name = '';
    if (key >= Qt.Key_A && key <= Qt.Key_Z || key >= Qt.Key_0 && key <= Qt.Key_9) name = String.fromCharCode(key);
    else if (key >= Qt.Key_F1 && key <= Qt.Key_F35) name = 'F' + (key - Qt.Key_F1 + 1);
    else {
        var names = {};
        names[Qt.Key_Space] = 'space'; names[Qt.Key_Return] = 'Return'; names[Qt.Key_Enter] = 'KP_Enter';
        names[Qt.Key_Tab] = 'Tab'; names[Qt.Key_Backtab] = 'Tab'; names[Qt.Key_Escape] = 'Escape';
        names[Qt.Key_Backspace] = 'BackSpace'; names[Qt.Key_Delete] = 'Delete'; names[Qt.Key_Insert] = 'Insert';
        names[Qt.Key_Home] = 'Home'; names[Qt.Key_End] = 'End'; names[Qt.Key_PageUp] = 'Prior'; names[Qt.Key_PageDown] = 'Next';
        names[Qt.Key_Left] = 'Left'; names[Qt.Key_Right] = 'Right'; names[Qt.Key_Up] = 'Up'; names[Qt.Key_Down] = 'Down';
        names[Qt.Key_Minus] = 'minus'; names[Qt.Key_Equal] = 'equal'; names[Qt.Key_Plus] = 'plus';
        names[Qt.Key_Comma] = 'comma'; names[Qt.Key_Period] = 'period'; names[Qt.Key_Slash] = 'slash';
        names[Qt.Key_Semicolon] = 'semicolon'; names[Qt.Key_Apostrophe] = 'apostrophe';
        names[Qt.Key_BracketLeft] = 'bracketleft'; names[Qt.Key_BracketRight] = 'bracketright';
        names[Qt.Key_Backslash] = 'backslash'; names[Qt.Key_QuoteLeft] = 'grave';
        name = names[key] || '';
    }
    if (modifiers & Qt.KeypadModifier && name !== 'KP_Enter') return ''; // Avoid ambiguous keypad bindings.
    return name ? mods.concat([name]).join(' + ') : '';
}
