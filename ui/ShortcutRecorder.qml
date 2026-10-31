import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell.Wayland
import "ShortcutKeys.js" as ShortcutKeys

RowLayout {
    id: root
    required property var recorderWindow
    property string value: ''
    property bool recording: false
    property string pending: ''
    property int pendingKey: 0
    property string hint: ''
    readonly property bool captureReady: inhibitor.active
    function cancel() { recording = false; pending = ''; hint = '' }
    onCaptureReadyChanged: if (!captureReady && recording) cancel()
    Connections {
        target: root.recorderWindow
        function onVisibleChanged() { if (!root.recorderWindow.visible) root.cancel() }
    }
    onVisibleChanged: if (!visible) cancel()
    onEnabledChanged: if (!enabled) cancel()
    ShortcutInhibitor {
        id: inhibitor
        window: root.recorderWindow
        enabled: root.recording
        onCancelled: root.cancel()
    }
    Timer { interval: 15000; running: root.recording; onTriggered: root.cancel() }
    Button {
        id: recordButton
        Layout.fillWidth: true
        text: root.recording ? (root.hint || (root.captureReady ? 'Press shortcut…' : 'Waiting for keyboard…')) : (root.value || 'Disabled')
        Accessible.name: 'Record shortcut: ' + (root.value || 'disabled')
        onClicked: {
            if (root.recording) root.cancel()
            else { root.recording = true; root.hint = ''; forceActiveFocus() }
        }
        onActiveFocusChanged: if (!activeFocus) root.cancel()
        Keys.onShortcutOverride: event => { if (root.recording) event.accepted = true }
        Keys.onPressed: event => {
            if (!root.recording) return
            event.accepted = true
            if (event.isAutoRepeat) return
            if (event.key === Qt.Key_Escape && event.modifiers === Qt.NoModifier) { root.cancel(); return }
            if (!root.captureReady) return
            const result = ShortcutKeys.combination(event.key, event.modifiers)
            if (result === null) return
            if (!result) { root.hint = 'Use modifiers + a supported key'; return }
            root.pending = result; root.pendingKey = event.key; root.hint = result
        }
        Keys.onReleased: event => {
            if (!root.recording) return
            event.accepted = true
            if (!event.isAutoRepeat && root.pending && event.key === root.pendingKey) {
                root.value = root.pending
                root.cancel()
            }
        }
    }
    Button {
        focusPolicy: Qt.NoFocus
        text: root.recording ? 'Cancel' : 'Clear'
        enabled: root.recording || root.value.length > 0
        onClicked: { if (root.recording) root.cancel(); else root.value = '' }
    }
}
