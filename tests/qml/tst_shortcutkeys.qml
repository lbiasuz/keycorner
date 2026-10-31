import QtQuick
import QtTest
import '../../ui/ShortcutKeys.js' as ShortcutKeys

TestCase {
    name: 'ShortcutKeys'
    function test_combinations() {
        compare(ShortcutKeys.combination(Qt.Key_P, Qt.MetaModifier | Qt.ControlModifier | Qt.ShiftModifier), 'SUPER + CTRL + SHIFT + P')
        compare(ShortcutKeys.combination(Qt.Key_F12, Qt.ControlModifier), 'CTRL + F12')
        compare(ShortcutKeys.combination(Qt.Key_Return, Qt.AltModifier), 'ALT + Return')
        compare(ShortcutKeys.combination(Qt.Key_Space, Qt.ControlModifier), 'CTRL + space')
        compare(ShortcutKeys.combination(Qt.Key_Backtab, Qt.ControlModifier | Qt.ShiftModifier), 'CTRL + SHIFT + Tab')
        compare(ShortcutKeys.combination(Qt.Key_Minus, Qt.MetaModifier), 'SUPER + minus')
    }
    function test_invalid_or_incomplete() {
        compare(ShortcutKeys.combination(Qt.Key_Control, Qt.ControlModifier), null)
        compare(ShortcutKeys.combination(Qt.Key_P, Qt.NoModifier), '')
        compare(ShortcutKeys.combination(Qt.Key_unknown, Qt.ControlModifier), '')
        compare(ShortcutKeys.combination(Qt.Key_1, Qt.ControlModifier | Qt.KeypadModifier), '')
        compare(ShortcutKeys.combination(Qt.Key_A, Qt.ControlModifier | Qt.GroupSwitchModifier), '')
    }
}
