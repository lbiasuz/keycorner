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

const first = '1'.repeat(24), second = '2'.repeat(24);
let profileConfig = c.normalize({backend:'vial',deviceId:first});
let profiles = c.saveKeyboard(profileConfig,{deviceId:first,name:'Example 40',layout:{keys:Array(47).fill({}),layers:10}});
profileConfig = c.normalize({...profileConfig,keyboards:profiles});
assert.equal(c.selectKeyboard(profileConfig,{id:first,vial:false}).backend,'vial');
assert.equal(c.selectKeyboard(profileConfig,{id:second,vial:true}).backend,'vial');
assert.equal(c.selectKeyboard(profileConfig,{id:second,vial:false}).backend,'standard');
profileConfig = c.normalize({...profileConfig,...c.selectKeyboard(profileConfig,{id:second,vial:false}),standardLayout:'ansi87'});
profiles = c.saveKeyboard(profileConfig,{deviceId:second,name:'Example TKL',layout:{keys:Array(87).fill({}),layers:1}});
assert.equal(Object.keys(profiles).length,2);
assert.equal(profiles[first].keys,47);
assert.equal(profiles[second].standardLayout,'ansi87');
assert.throws(()=>c.saveKeyboard(profileConfig,{deviceId:first,name:'Wrong',layout:{keys:[{}],layers:1}}));
for (const keyboards of [null,[],{'../bad':profiles[first]},{[first]:{...profiles[first],backend:'bad'}},{[first]:{...profiles[first],keys:2048}},{[first]:{...profiles[first],layers:0}}]) assert.throws(()=>c.normalize({keyboards}));
assert.equal(JSON.stringify(c.normalize({...profileConfig,keyboards:profiles}).keyboards),JSON.stringify(profiles));
console.log('Keyboard profiles, selection, isolation and validation passed.');
