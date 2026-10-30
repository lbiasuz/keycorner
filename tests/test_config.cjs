const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const source = fs.readFileSync('Config.js','utf8');
const c = vm.createContext({}); vm.runInContext(source,c);
assert.equal(c.normalize({}).opacity,.78);
assert.equal(c.normalize({opacity:0}).opacity,.15);
assert.equal(c.normalize({keySize:100}).keySize,60);
assert.equal(c.normalize({previewLayer:2.8}).previewLayer,2);
for (const bad of [null,[],{version:2},{shown:'true'},{positionLocked:'true'},{opacity:NaN},{corner:'center'}]) {
    assert.throws(() => c.normalize(bad));
}
const saved = c.normalize({opacity:.63,corner:'top-left',clickThrough:true});
assert.equal(JSON.stringify(c.normalize(JSON.parse(JSON.stringify(saved)))),JSON.stringify(saved));
console.log('Settings validation and persistence round-trip passed.');

assert.equal(c.normalize({}).positionLocked, false);
assert.equal(c.normalize({positionLocked:true,clickThrough:false}).positionLocked, true);
assert.equal(c.normalize({positionLocked:false,clickThrough:true}).positionLocked, false);

assert.equal(c.defaults().backend, 'standard');
assert.equal(c.normalize({version:1}).backend, 'vial');
assert.equal(c.normalize({backend:'standard',deviceId:'a'.repeat(24)}).backend,'standard');
for (const value of [{backend:'bad'},{deviceId:'../event0'},{standardLayout:'bad'}]) assert.throws(() => c.normalize(value));
