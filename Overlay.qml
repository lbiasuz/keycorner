import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Hyprland
import QtQuick.Controls as Controls
import "Config.js" as Config

Item {
    id: root
    property var shell: null
    property var manifest: null
    property string omarchyPath: ''
    property var config: Config.defaults()
    property bool configReady: false
    property bool settingsInitialized: false
    property bool sessionClosed: false
    property int retryGeneration: 0
    property string configError: ''
    property bool settingsOpen: false
    readonly property string configPath: (Quickshell.env('XDG_CONFIG_HOME') || Quickshell.env('HOME') + '/.config') + '/omarchy/keycorner.json'
    function loadConfig() {
        try { config = Config.normalize(JSON.parse(configFile.text())); configError = ''; configReady = true }
        catch (error) { configError = String(error); configReady = true }
    }
    function updateConfig(patch) {
        if (configError) return
        var next = Object.assign({}, config, patch)
        try { config = Config.normalize(next); configFile.setText(JSON.stringify(config, null, 2) + '\n') }
        catch (error) { console.warn('Invalid Keycorner settings: ' + error) }
    }
    function open(payloadJson) {
        var payload = {}
        try { payload = JSON.parse(payloadJson || '{}') } catch (e) {}
        if (payload.settings) settingsOpen = true
        else { sessionClosed = false; updateConfig({shown: true}) }
    }
    function close() { updateConfig({shown: false}) }
    function dismiss() { close() }
    function toggle() { const next = !shown || paused; sessionClosed = false; updateConfig({shown: next, paused: !next}) }
    Process {
        command: ['python', '-B', Qt.resolvedUrl('keycorner/config_file.py').toString().replace('file://','')]
        running: true
        onExited: (exitCode, exitStatus) => {
            if (exitCode === 0) root.settingsInitialized = true
            else { root.configError = 'Could not initialize settings'; root.settingsOpen = true }
        }
    }
    FileView {
        id: configFile
        path: root.settingsInitialized ? root.configPath : ''
        watchChanges: true
        atomicWrites: true
        onLoaded: root.loadConfig()
        onFileChanged: reload()
        onLoadFailed: { root.configError = 'Could not read settings; run the installer or repair keycorner.json'; root.configReady = true }
        onSaveFailed: { root.configError = 'Could not save settings' }
    }
    property var shortcuts: ({toggle: '', toggleSettings: '', pause: ''})
    property string shortcutMessage: ''
    property bool shortcutError: false
    readonly property bool shortcutsBusy: shortcutProcess.running
    function manageShortcuts(values) {
        if (shortcutProcess.running) return
        shortcutMessage = ''
        shortcutProcess.response = ''
        shortcutProcess.applying = values !== undefined
        shortcutProcess.command = ['python', '-B', Qt.resolvedUrl('keycorner/shortcuts.py').toString().replace('file://', '')]
            .concat(values === undefined ? [] : ['apply', JSON.stringify(values)])
        shortcutProcess.running = true
    }
    onSettingsOpenChanged: if (settingsOpen) manageShortcuts()
    Process {
        id: shortcutProcess
        property string response: ''
        property bool applying: false
        stdout: StdioCollector { onStreamFinished: shortcutProcess.response = text }
        onExited: (exitCode, exitStatus) => {
            try {
                const result = JSON.parse(response)
                root.shortcutError = !result.ok
                if (result.ok) {
                    root.shortcuts = result.shortcuts
                    root.shortcutMessage = applying ? 'Shortcuts applied' : ''
                } else root.shortcutMessage = result.error
            } catch (error) {
                root.shortcutError = true
                root.shortcutMessage = 'Could not update shortcuts'
            }
        }
    }
    SettingsWindow { controller: root; visible: root.settingsOpen }
    property var layout: ({keys: [], layers: 0, width: 14.25, height: 6})
    property var palette: ({background: '#faf4ed', foreground: '#575279', accent: '#56949f', muted: '#cecacd', lighter_background: '#f2e9e1', dark_foreground: '#9893a5'})
    property var pressed: []
    readonly property int preview: Math.min(config.previewLayer, Math.max(0, layout.layers-1))
    property int estimate: -1
    readonly property bool automatic: config.suggestLayers
    readonly property bool paused: config.paused
    readonly property bool shown: config.shown && !sessionClosed
    readonly property bool passthrough: config.clickThrough
    readonly property bool positionLocked: config.positionLocked
    function updatePointerPolicy() {
        if (!panel.backingWindowVisible) return
        // Hyprland hit-tests pinned floating windows by their bounding box.
        // Exclude this window from that search as well as emptying its input mask.
        Hyprland.dispatch('hl.dsp.window.set_prop({window="title:^Keycorner Overlay$",prop="no_focus",value="' + (passthrough ? 'true' : 'false') + '"})')
    }
    onPassthroughChanged: updatePointerPolicy()
    readonly property string corner: config.corner
    readonly property string monitor: config.monitor
    readonly property real unit: Math.min((panel.width - 32) / layout.width, (panel.height - 72) / layout.height)
    readonly property real translucency: config.opacity
    readonly property color surface: color('background', '#faf4ed')
    readonly property color keySurface: color('lighter_background', '#f2e9e1')
    readonly property color translucentKey: Qt.rgba(keySurface.r, keySurface.g, keySurface.b, translucency)
    property string status: 'Connecting to Crosses 54…'
    property bool connected: false
    readonly property int activeLayer: automatic && estimate >= 0 ? estimate : preview
    function color(name, fallback) { return palette[name] || fallback }
    function pause() {
        updateConfig({paused: !paused})
        pressed = []; estimate = -1
        if (paused) status = 'Paused · input access stopped'
    }
    function consume(line) {
        try {
            const msg = JSON.parse(line)
            if (!captureEnabled) return
            if (msg.type === 'layout') {
                layout = msg.layout; connected = true
                if (bridge.running) bridge.write(root.automatic ? 'suggest:1\n' : 'suggest:0\n')
                status = 'Live key output · layer telemetry unavailable'
            } else if (msg.type === 'state') {
                pressed = msg.pressed
                estimate = msg.estimate === null ? -1 : msg.estimate
            } else if (msg.type === 'error') {
                status = msg.message; connected = false; pressed = []; estimate = -1
            }
        } catch (error) { status = 'Could not read keyboard state'; connected = false }
    }
    // Hyprland clears activewindow before presenting session-lock surfaces.
    // No application focus is deliberately treated as unsafe, including launchers
    // and empty workspaces. Only the public lock IPC is consulted on re-entry.
    property bool rawFocusBlocked: false
    readonly property bool applicationFocused: Hyprland.activeToplevel !== null && !rawFocusBlocked
    Connections {
        target: Hyprland
        function onRawEvent(event) {
            // activeToplevel can stay cached on lock: use the explicit IPC clear.
            if (event.name === 'openwindow' && event.data.endsWith(',Keycorner Overlay')) root.updatePointerPolicy()
            if (event.name === 'activewindowv2') root.rawFocusBlocked = event.data.trim() === ''
        }
    }
    property bool unlockedConfirmed: false
    property int focusEpoch: 0
    function checkSession() {
        if (applicationFocused && !lockCheck.running) {
            lockCheck.epoch = focusEpoch
            lockCheck.answer = ''
            lockCheck.running = true
        }
    }
    onApplicationFocusedChanged: {
        focusEpoch++
        unlockedConfirmed = false
        if (applicationFocused) checkSession()
    }
    Component.onCompleted: checkSession()
    Process {
        id: lockCheck
        property int epoch: -1
        property string answer: ''
        command: ['omarchy-shell', 'lock', 'isLocked']
        stdout: StdioCollector { onStreamFinished: lockCheck.answer = text.trim() }
        onExited: (exitCode, exitStatus) => {
            if (epoch !== root.focusEpoch) { root.checkSession(); return }
            root.unlockedConfirmed = exitCode === 0 && answer === 'false' && root.applicationFocused
        }
    }
    readonly property bool sessionSafe: applicationFocused && unlockedConfirmed
    readonly property bool captureEnabled: configReady && !configError && shown && !panel.minimized && !paused && sessionSafe
    onCaptureEnabledChanged: {
        pressed = []; estimate = -1
        if (!captureEnabled) status = !sessionSafe ? 'Session protected · input access stopped' : paused ? 'Paused · input access stopped' : 'Hidden · input access stopped'
    }
    // Theme loading must not depend on the keyboard connection or capture state.
    property bool themePending: false
    function refreshTheme() {
        if (themeReader.running) themePending = true
        else { themePending = false; themeReader.running = true }
    }
    FileView {
        path: (Quickshell.env('HOME') || '') + '/.local/state/omarchy/current/theme/colors.toml'
        watchChanges: true
        printErrors: false
        onLoaded: root.refreshTheme()
        onLoadFailed: root.refreshTheme()
        onFileChanged: reload()
    }
    Process {
        id: themeReader
        command: ['python', '-B', Qt.resolvedUrl('keycorner/bridge.py').toString().replace('file://',''), '--theme-only']
        stdout: StdioCollector {
            onStreamFinished: {
                try { root.palette = JSON.parse(text).theme } catch (error) {}
            }
        }
        onExited: { if (root.themePending) root.refreshTheme() }
    }
    onAutomaticChanged: if (bridge.running) bridge.write(root.automatic ? 'suggest:1\n' : 'suggest:0\n')
    Process {
        id: bridge
        command: ['python', '-B', '-u', Qt.resolvedUrl('keycorner/bridge.py').toString().replace('file://','')]
        running: root.captureEnabled && root.retryGeneration >= 0
        stdinEnabled: true
        onStarted: bridge.write(root.automatic ? 'suggest:1\n' : 'suggest:0\n')
        stdout: SplitParser { onRead: data => root.consume(data) }
        onExited: {
            root.pressed = []; root.estimate = -1; root.connected = false
            if (root.captureEnabled) { root.status = 'Bridge stopped · retrying'; retry.restart() }
        }
    }
    Timer { id: retry; interval: 2000; onTriggered: { if (root.captureEnabled) root.retryGeneration++ } }
    IpcHandler {
        target: 'keycorner'
        function toggle(): void { root.toggle() }
        function pause(): void { root.pause() }
        function settings(): void { root.settingsOpen = true }
        function toggleSettings(): void { root.settingsOpen = !root.settingsOpen }
        function closeSettings(): void { root.settingsOpen = false }
        function state(): string { return JSON.stringify({config:root.config, error:root.configError, connected:root.connected, capture:root.captureEnabled, sessionSafe:root.sessionSafe, settingsOpen:root.settingsOpen, applicationFocused:root.applicationFocused, layers:root.layout.layers, keys:root.layout.keys.length}) }
        function configure(json: string): void {
            try { root.updateConfig(JSON.parse(json)) } catch (error) { console.warn('Invalid Keycorner settings') }
        }
        function reload(): void { configFile.reload() }
        function quit(): void { root.close() }
        function layer(value: int): void { root.updateConfig({previewLayer:value,suggestLayers:false}) }
        function auto(): void { root.updateConfig({suggestLayers:true}) }
        function corner(value: string): void { root.updateConfig({corner:value}) }
        function monitor(value: string): void { root.updateConfig({monitor:value}) }
        function opacity(value: real): void { root.updateConfig({opacity:value}) }
        function scale(value: real): void { root.updateConfig({keySize:value}) }
        function passthrough(): void { root.updateConfig({clickThrough:!root.passthrough}) }
    }
    FloatingWindow {
        id: panel
        title: "Keycorner Overlay"
        // Closing a backing window during hot reload must not persist a hidden state.
        onClosed: root.sessionClosed = true
        visible: root.configReady && root.shown && root.sessionSafe
        screen: Quickshell.screens.find(s => s.name === root.monitor) || Quickshell.screens[0]
        implicitWidth: root.layout.width * root.config.keySize + 32
        implicitHeight: root.layout.height * root.config.keySize + 72
        minimumSize: Qt.size(400, 200)
        color: 'transparent'
        // The input mask also protects against pointer delivery directly to Qt.
        mask: Region { width: root.passthrough ? 0 : panel.width; height: root.passthrough ? 0 : panel.height }
        onBackingWindowVisibleChanged: if (backingWindowVisible) Qt.callLater(root.updatePointerPolicy)
        Rectangle {
            id: card
            anchors.fill: parent
            radius: 12
            color: Qt.rgba(root.surface.r, root.surface.g, root.surface.b, root.translucency)
            border.color: root.color('muted','#cecacd')
            border.width: 1
            MouseArea {
                anchors.fill: parent
                enabled: !root.positionLocked && !root.passthrough
                cursorShape: enabled ? Qt.SizeAllCursor : Qt.ArrowCursor
                onPressed: if (!root.positionLocked) panel.startSystemMove()
            }
            Row {
                opacity: root.passthrough ? 0.45 : 1
                anchors { right: parent.right; rightMargin: 14; top: parent.top; topMargin: 9 }
                spacing: 6
                Rectangle {
                    width: 25; height: 25; radius: 4; color: root.translucentKey
                    Item {
                        anchors.centerIn: parent
                        width: 15; height: 15
                        Repeater {
                            model: 3
                            Item {
                                required property int index
                                y: index * 6; width: 15; height: 3
                                Rectangle { anchors.verticalCenter: parent.verticalCenter; width: 15; height: 1; color: root.palette.foreground }
                                Rectangle { x: parent.index === 1 ? 9 : 3; width: 3; height: 3; radius: 1; color: root.palette.foreground }
                            }
                        }
                    }
                    MouseArea { id: gearMouse; anchors.fill: parent; hoverEnabled: true; onClicked: root.settingsOpen = true }
                    Controls.ToolTip.visible: gearMouse.containsMouse
                    Controls.ToolTip.text: 'Settings'
                }
                Rectangle {
                    width: 25; height: 25; radius: 4; color: root.translucentKey
                    Text { textFormat: Text.PlainText; anchors.centerIn: parent; text: root.paused ? '▶' : 'Ⅱ'; color: root.palette.foreground; font.pixelSize: 10 }
                    MouseArea { anchors.fill: parent; onClicked: root.pause() }
                }
                Rectangle {
                    width: 25; height: 25; radius: 4; color: root.translucentKey
                    Text { textFormat: Text.PlainText; anchors.centerIn: parent; text: '×'; color: root.palette.foreground; font.pixelSize: 17 }
                    MouseArea { anchors.fill: parent; onClicked: root.close() }
                }
            }
            Row {
                opacity: root.passthrough ? 0.45 : 1
                x: 16; y: 10; spacing: 4
                Repeater {
                    model: root.layout.layers
                    Rectangle {
                        required property int index
                        width: 28; height: 24; radius: 5
                        color: root.activeLayer === index ? root.palette.accent : root.translucentKey
                        Text { textFormat: Text.PlainText; anchors.centerIn: parent; text: 'L'+index; color: root.activeLayer === index ? root.palette.background : root.palette.foreground; font { family: 'monospace'; pixelSize: 11; bold: true } }
                        MouseArea { anchors.fill: parent; onClicked: { root.updateConfig({previewLayer:index,suggestLayers:false}) } }
                    }
                }
                Rectangle {
                    width: 25; height: 24; radius: 5
                    color: root.translucentKey
                    border.width: root.automatic ? 1 : 0; border.color: root.palette.accent
                    Text { textFormat: Text.PlainText; anchors.centerIn: parent; text: '≈'; color: root.palette.foreground; font.pixelSize: 10 }
                    MouseArea { anchors.fill: parent; onClicked: root.updateConfig({suggestLayers:!root.automatic}) }
                }
            }
            Item {
                x: (panel.width - width) / 2; y: 52 + (panel.height - 72 - height) / 2
                width: root.layout.width * root.unit; height: root.layout.height * root.unit
                Repeater {
                    model: root.layout.keys
                    Rectangle {
                        required property var modelData
                        readonly property int code: modelData.usages[root.activeLayer] || 0
                        readonly property bool down: code > 0 && root.pressed.includes(code)
                        x: modelData.x * root.unit; y: modelData.y * root.unit
                        width: modelData.w * root.unit - 3; height: modelData.h * root.unit - 3
                        radius: 4
                        color: down ? root.palette.accent : root.translucentKey
                        border.width: 1
                        border.color: down ? root.palette.accent : root.color('muted','#cecacd')
                        Text { textFormat: Text.PlainText;
                            anchors { fill: parent; margins: 2 }
                            horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
                            text: modelData.labels[root.activeLayer] || '—'
                            color: parent.down ? root.palette.background : root.palette.foreground
                            font { family: 'monospace'; pixelSize: root.unit * 0.29; bold: parent.down }
                            fontSizeMode: Text.Fit; minimumPixelSize: 7
                        }
                    }
                }
            }
            Text { textFormat: Text.PlainText;
                anchors { right: parent.right; bottom: parent.bottom; margins: 4 }
                visible: !root.positionLocked && !root.passthrough
                text: '◢'; font.pixelSize: 17; color: root.palette.foreground; opacity: 0.65
                MouseArea {
                    anchors.fill: parent
                    anchors.margins: -5
                    cursorShape: Qt.SizeFDiagCursor
                    enabled: !root.positionLocked && !root.passthrough
                    onPressed: if (!root.positionLocked) panel.startSystemResize(Qt.RightEdge | Qt.BottomEdge)
                }
            }
        }
    }
}
