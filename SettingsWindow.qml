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
    Pane {
        anchors.fill: parent
        padding: 24
        palette.window: controller.palette.background
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
                Label { text: 'Keycorner'; font.pixelSize: 24; font.bold: true }
                Label { text: 'Drag the keyboard to move it; use its bottom-right grip to resize.'; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                Label { text: controller.configError; visible: text.length > 0; color: '#b4637a'; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                RowLayout {
                    Switch { text: 'Show overlay'; checked: controller.config.shown; onToggled: controller.updateConfig({shown:checked}) }
                    Switch { text: 'Pause input'; checked: controller.config.paused; onToggled: controller.updateConfig({paused:checked}) }
                }
                GridLayout {
                    columns: 2; columnSpacing: 20; rowSpacing: 12
                    Layout.fillWidth: true
                    Label { text: 'Monitor' }
                    ComboBox {
                        Layout.fillWidth: true
                        model: ['Automatic'].concat(Quickshell.screens.map(s => s.name))
                        currentIndex: Math.max(0, model.indexOf(controller.config.monitor))
                        onActivated: controller.updateConfig({monitor:currentIndex === 0 ? '' : currentText})
                    }
                    Label { text: 'Opacity · ' + Math.round(controller.config.opacity * 100) + '%' }
                    Slider {
                        Layout.fillWidth: true; from: 0.15; to: 1; stepSize: 0.01
                        value: controller.config.opacity
                        onMoved: controller.updateConfig({opacity:value})
                    }
                    Label { text: 'Key size · ' + controller.config.keySize + ' px' }
                    Slider {
                        Layout.fillWidth: true; from: 28; to: 60; stepSize: 1
                        value: controller.config.keySize
                        onMoved: controller.updateConfig({keySize:value})
                    }
                    Label { text: 'Reference layer' }
                    SpinBox {
                        from: 0; to: Math.max(0,controller.layout.layers-1)
                        value: controller.preview
                        onValueModified: controller.updateConfig({previewLayer:value})
                    }
                }
                Switch { text: 'Suggest a layer from key output'; checked: controller.config.suggestLayers; onToggled: controller.updateConfig({suggestLayers:checked}) }
                Switch { text: 'Let mouse clicks pass through the overlay'; checked: controller.config.clickThrough; onToggled: controller.updateConfig({clickThrough:checked}) }
                Label {
                    text: 'Exact layer transitions need future firmware support. Suggestions are estimates; previews show a saved layout.'
                    wrapMode: Text.WordWrap; Layout.fillWidth: true; opacity: 0.75
                }
                Label {
                    text: 'Use the keyboard icon in the bar: click to show/hide, right-click for settings, middle-click to pause. Hiding stops input access.'
                    wrapMode: Text.WordWrap; Layout.fillWidth: true; opacity: 0.75
                }
                RowLayout {
                    Layout.fillWidth: true
                    Label { text: controller.status; Layout.fillWidth: true }
                    Button { text: 'Done'; onClicked: controller.settingsOpen = false }
                }
            }
        }
    }
}
