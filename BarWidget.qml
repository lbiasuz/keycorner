import QtQuick
import Quickshell
import qs.Ui

BarWidget {
    id: root
    moduleName: 'lbiasuz.keycorner'
    implicitWidth: 30
    implicitHeight: barSize
    Text {
        anchors.centerIn: parent
        text: '󰌌'
        font.family: root.bar ? root.bar.fontFamily : 'monospace'
        font.pixelSize: 17
        color: root.bar ? root.bar.barForeground : '#575279'
    }
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        onClicked: mouse => {
            const action = mouse.button === Qt.RightButton ? 'settings' : mouse.button === Qt.MiddleButton ? 'pause' : 'toggle'
            Quickshell.execDetached(['omarchy-shell', 'keycorner', action])
        }
    }
}
