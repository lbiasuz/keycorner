import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell

FloatingWindow {
    id: window
    required property var controller
    title: 'Keycorner Settings'
    implicitWidth: 510
    implicitHeight: 700
    minimumSize: Qt.size(460, 500)
    color: controller.palette.background
    onVisibleChanged: if (!visible) controller.settingsOpen = false
    Dialog {
        id: permissionDialog
        anchors.centerIn: parent
        width: Math.min(460, window.width - 32)
        modal: true; title: 'Enable keyboard access'
        palette.windowText: controller.palette.foreground
        palette.text: controller.palette.foreground
        palette.buttonText: controller.palette.foreground
        palette.button: controller.palette.lighter_background
        background: Rectangle { color: controller.palette.lighter_background; border.color: controller.palette.muted; radius: 6 }
        visible: controller.permissionPlan !== null
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAccepted: controller.devicePermission('install')
        onRejected: controller.permissionPlan = null
        contentItem: ColumnLayout {
            Label { textFormat: Text.PlainText; Layout.fillWidth: true; wrapMode: Text.WordWrap; text: 'Grant the active local session access to ' + (controller.permissionPlan ? controller.permissionPlan.name : '') + '. Administrator authentication is required.' }
            Label { textFormat: Text.PlainText; Layout.fillWidth: true; wrapMode: Text.WrapAnywhere; text: controller.permissionPlan ? controller.permissionPlan.path : '' }
            Label { textFormat: Text.PlainText; Layout.fillWidth: true; wrapMode: Text.WrapAnywhere; font.family: 'monospace'; font.pixelSize: 10; text: controller.permissionPlan ? controller.permissionPlan.rule : '' }
        }
    }
    Pane {
        anchors.fill: parent
        padding: 24
        palette.window: controller.palette.background
        palette.light: controller.palette.lighter_background
        palette.midlight: controller.palette.lighter_background
        palette.dark: controller.palette.background
        palette.mid: controller.palette.muted
        palette.shadow: controller.palette.background
        palette.alternateBase: controller.palette.lighter_background
        palette.placeholderText: controller.palette.dark_foreground
        palette.base: controller.palette.lighter_background
        palette.button: controller.palette.lighter_background
        palette.text: controller.palette.foreground
        palette.windowText: controller.palette.foreground
        palette.buttonText: controller.palette.foreground
        palette.highlight: controller.palette.accent
        palette.highlightedText: controller.palette.background
        background: Rectangle { color: controller.palette.background }
        ScrollView {
            anchors.fill: parent
            contentWidth: availableWidth
            ColumnLayout {
                width: parent.width
                spacing: 12
                Label { textFormat: Text.PlainText; text: 'Keycorner'; font.pixelSize: 24; font.bold: true }
                Label { textFormat: Text.PlainText; text: controller.configError; visible: text.length > 0; color: controller.palette.red || '#b4637a'; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                Label { textFormat: Text.PlainText; text: 'Keyboard setup' }
                RowLayout {
                    ComboBox {
                        Layout.fillWidth: true; enabled: !controller.deviceBusy
                        model: ['Select a keyboard'].concat(controller.keyboardChoices.map(d => d.name + ' · ' + d.connection + (controller.config.keyboards[d.id] ? ' · Saved' : '')))
                        currentIndex: Math.max(0, controller.keyboardChoices.findIndex(d => d.id === controller.config.deviceId) + 1)
                        onActivated: if (currentIndex > 0) {
                            const device = controller.keyboardChoices[currentIndex-1]
                            controller.selectKeyboard(device)
                        }
                    }
                    Button { text: 'Refresh'; onClicked: controller.scanDevices() }
                }
                RowLayout {
                    ComboBox {
                        Layout.fillWidth: true; enabled: !controller.deviceBusy
                        model: controller.selectedKeyboard && controller.selectedKeyboard.vial ? ['Standard', 'Vial layout'] : ['Standard']
                        currentIndex: controller.config.backend === 'vial' && model.length > 1 ? 1 : 0
                        onActivated: controller.updateConfig({backend:currentIndex === 1 ? 'vial' : 'standard'})
                    }
                    Button {
                        text: 'Enable access'; enabled: !!controller.selectedKeyboard && !controller.deviceBusy
                        onClicked: controller.devicePermission('preview')
                    }
                }
                ComboBox {
                    visible: controller.config.backend === 'standard'; Layout.fillWidth: true; enabled: !controller.deviceBusy
                    model: ['US ANSI 104', 'US ANSI TKL', 'US ANSI 60%']
                    property var presets: ['ansi104','ansi87','ansi60']
                    currentIndex: presets.indexOf(controller.config.standardLayout)
                    onActivated: controller.updateConfig({standardLayout:presets[currentIndex]})
                }
                RowLayout {
                    Button {
                        text: controller.savedKeyboard ? 'Read layout & save' : 'Add keyboard'
                        enabled: !!controller.selectedKeyboard && !controller.deviceBusy && controller.sessionSafe
                        onClicked: controller.setupKeyboard()
                    }
                    Button { text: 'Forget'; visible: !!controller.savedKeyboard; enabled: !controller.deviceBusy; onClicked: controller.forgetKeyboard() }
                }
                Label {
                    textFormat: Text.PlainText; Layout.fillWidth: true; wrapMode: Text.WordWrap
                    text: controller.deviceMessage || (!controller.selectedKeyboard ? 'Keyboard not selected or disconnected' :
                        controller.config.backend === 'vial' && !controller.selectedKeyboard.vial ? 'Use Standard mode for this keyboard' :
                        (controller.config.backend === 'standard' ? controller.selectedKeyboard.standardAccessible : controller.selectedKeyboard.vialAccessible) ? (controller.savedKeyboard ? (controller.profileMatchesSelection ? 'Saved · ' + controller.savedKeyboard.keys + ' keys · ' + controller.savedKeyboard.layers + ' layers' : 'Unsaved changes') : 'Ready to add keyboard') : 'Enable access to continue setup')
                }
                Label {
                    textFormat: Text.PlainText; Layout.fillWidth: true; wrapMode: Text.WordWrap
                    visible: controller.captureEnabled && !controller.connected
                    text: controller.status
                }
                RowLayout {
                    Switch { text: 'Show overlay'; checked: controller.config.shown; onToggled: controller.updateConfig({shown:checked}) }
                    Switch { text: 'Pause input'; checked: controller.config.paused; onToggled: controller.updateConfig({paused:checked}) }
                }
                GridLayout {
                    columns: 2; columnSpacing: 20; rowSpacing: 12
                    Layout.fillWidth: true
                    Label { textFormat: Text.PlainText; text: 'Monitor' }
                    ComboBox {
                        Layout.fillWidth: true
                        model: ['Automatic'].concat(Quickshell.screens.map(s => s.name))
                        currentIndex: Math.max(0, model.indexOf(controller.config.monitor))
                        onActivated: controller.updateConfig({monitor:currentIndex === 0 ? '' : currentText})
                    }
                    Label { textFormat: Text.PlainText; text: 'Opacity · ' + Math.round(controller.config.opacity * 100) + '%' }
                    Slider {
                        Layout.fillWidth: true; from: 0.15; to: 1; stepSize: 0.01
                        value: controller.config.opacity
                        onMoved: controller.updateConfig({opacity:value})
                    }
                    Label { textFormat: Text.PlainText; text: 'Key size · ' + controller.config.keySize + ' px' }
                    Slider {
                        Layout.fillWidth: true; from: 28; to: 60; stepSize: 1
                        value: controller.config.keySize
                        onMoved: controller.updateConfig({keySize:value})
                    }
                    Label { textFormat: Text.PlainText; text: 'Preview layer'; visible: controller.config.backend === 'vial' }
                    SpinBox {
                        visible: controller.config.backend === 'vial'
                        from: 0; to: Math.max(0,controller.layout.layers-1)
                        value: controller.preview
                        onValueModified: controller.updateConfig({previewLayer:value})
                    }
                }
                Switch { visible: controller.config.backend === 'vial'; text: 'Suggest layer'; checked: controller.config.suggestLayers; onToggled: controller.updateConfig({suggestLayers:checked}) }
                Switch { text: 'Modifier badges'; checked: controller.config.showModifierBadges; onToggled: controller.updateConfig({showModifierBadges:checked}) }
                Switch { visible: controller.config.backend === 'vial'; text: 'Highlight modifier holds'; checked: controller.config.highlightModifierHolds; onToggled: controller.updateConfig({highlightModifierHolds:checked}) }
                Switch { text: 'Lock position'; checked: controller.config.positionLocked; onToggled: controller.updateConfig({positionLocked:checked}) }
                Switch { text: 'Click through overlay'; checked: controller.config.clickThrough; onToggled: controller.updateConfig({clickThrough:checked}) }
                Label { textFormat: Text.PlainText; text: 'Keyboard shortcuts'; font.bold: true; font.pixelSize: 18 }
                Label { textFormat: Text.PlainText; text: 'Show / hide keyboard' }
                ShortcutRecorder {
                    id: overlayShortcut
                    Layout.fillWidth: true; enabled: !controller.shortcutsBusy
                    value: controller.shortcuts.toggle
                    recorderWindow: window
                }
                Label { textFormat: Text.PlainText; text: 'Open / close settings' }
                ShortcutRecorder {
                    id: settingsShortcut
                    Layout.fillWidth: true; enabled: !controller.shortcutsBusy
                    value: controller.shortcuts.toggleSettings
                    recorderWindow: window
                }
                Label { textFormat: Text.PlainText; text: 'Pause / resume input' }
                ShortcutRecorder {
                    id: pauseShortcut
                    Layout.fillWidth: true; enabled: !controller.shortcutsBusy
                    value: controller.shortcuts.pause
                    recorderWindow: window
                }
                Connections {
                    target: controller
                    function onShortcutsChanged() {
                        overlayShortcut.value = controller.shortcuts.toggle
                        settingsShortcut.value = controller.shortcuts.toggleSettings
                        pauseShortcut.value = controller.shortcuts.pause
                    }
                }
                RowLayout {
                    Button {
                        text: controller.shortcutsBusy ? 'Applying…' : 'Apply shortcuts'
                        enabled: !controller.shortcutsBusy && !overlayShortcut.recording && !settingsShortcut.recording && !pauseShortcut.recording
                        onClicked: controller.manageShortcuts({toggle:overlayShortcut.value, toggleSettings:settingsShortcut.value, pause:pauseShortcut.value})
                    }
                    Button {
                        text: 'Defaults'; enabled: !controller.shortcutsBusy && !overlayShortcut.recording && !settingsShortcut.recording && !pauseShortcut.recording
                        onClicked: {
                            overlayShortcut.value = 'SUPER + CTRL + SHIFT + K'
                            settingsShortcut.value = 'SUPER + CTRL + SHIFT + O'
                            pauseShortcut.value = 'SUPER + CTRL + SHIFT + P'
                        }
                    }
                }
                Label {
                    textFormat: Text.PlainText; text: controller.shortcutMessage
                    visible: text.length > 0; wrapMode: Text.WordWrap; Layout.fillWidth: true
                    color: controller.shortcutError ? (controller.palette.red || '#b4637a') : controller.palette.foreground
                }
                RowLayout {
                    Layout.fillWidth: true
                    Item { Layout.fillWidth: true }
                    Button { text: 'Done'; onClicked: controller.settingsOpen = false }
                }
            }
        }
    }
}
