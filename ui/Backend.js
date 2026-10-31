// All production helpers run through the native executable.
var engine = 'rust'
function command(rootUrl, action, args) {
    var root = rootUrl.toString().replace('file://', '')
    return [root + 'bin/keycorner', action].concat(args || [])
}
