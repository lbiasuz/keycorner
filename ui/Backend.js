// The installer changes this constant for an explicit Python comparison install.
var engine = 'rust'
function command(rootUrl, action, args) {
    var root = rootUrl.toString().replace('file://', '')
    if (engine === 'rust' || action === 'inspect') return [root + 'bin/keycorner', action].concat(args || [])
    var scripts = {'bridge':'bridge.py', 'theme':'bridge.py', 'devices':'keyboard_devices.py',
                   'config-init':'config_file.py', 'shortcuts':'shortcuts.py', 'permissions':'permissions.py'}
    return ['python', '-B', '-u', root + 'reference/python/keycorner/' + scripts[action]]
        .concat(action === 'theme' ? ['--theme-only'] : (args || []))
}
