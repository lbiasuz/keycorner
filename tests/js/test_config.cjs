const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const source = fs.readFileSync('ui/Config.js','utf8');
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

assert.equal(c.normalize({}).showModifierBadges,true);
assert.equal(c.normalize({}).highlightModifierHolds,true);
assert.equal(c.normalize({showModifierBadges:false,highlightModifierHolds:false}).highlightModifierHolds,false);
assert.throws(()=>c.normalize({highlightModifierHolds:'true'}));

const fresh = c.defaults();
const vial = {id:first,name:'Unknown Vial',vial:true,keyboardReports:true,vialAccessible:true};
const ordinary = {id:second,name:'Ordinary',vial:false};
assert.equal(c.onboarding(fresh,[]).action,'choose');
assert.equal(c.onboarding(fresh,[vial,ordinary]).device.id,first);
assert.equal(c.onboarding(fresh,[vial,{...vial,id:second}]).action,'choose');
const selected = c.normalize({...fresh,...c.selectKeyboard(fresh,vial)});
assert.equal(c.onboarding(selected,[vial]).action,'inspect');
assert.equal(c.onboarding(selected,[{...vial,vialAccessible:false}]).action,'permission');
assert.equal(c.onboarding(selected,[{...vial,keyboardReports:false}]).action,'unsupported');
assert.equal(c.onboarding(selected,[ordinary]).action,'wait'); // Never hijack an explicit selection.
assert.equal(c.onboarding({...selected,backend:'standard'},[{...vial,standardAccessible:true}]).action,'standard');
assert.equal(c.onboarding({...selected,keyboards:{[first]:{backend:'vial',standardLayout:'ansi104'}}},[vial]).action,'ready');
console.log('Automatic onboarding, ambiguity, permission and reconnect decisions passed.');
assert.equal(c.onboarding({...selected,keyboards:{[first]:{backend:'vial',standardLayout:'ansi104'}}},[{...vial,vialAccessible:false}]).action,'permission');
assert.equal(c.onboarding({...selected,backend:'standard'},[{...vial,standardAccessible:false}]).action,'permission');
assert.equal(c.onboarding(fresh,[ordinary]).device.id,second);
assert.equal(c.onboarding({...selected,backend:'standard',keyboards:{[first]:{backend:'standard',standardLayout:'ansi104'}}},[{...vial,standardAccessible:true}]).action,'ready');

// Every selectable preset survives saving/reselection and has a valid geometry.
for (const p of c.standardPresets) {
    const layout = JSON.parse(fs.readFileSync(`data/${p.id}.json`, 'utf8'));
    const id = '0123456789abcdef01234567';
    const cfg = c.normalize({backend:'standard',deviceId:id,standardLayout:p.id,keyboards:{[id]:{name:'Synthetic keyboard',backend:'standard',standardLayout:p.id,keys:layout.keys.length,layers:1}}});
    const roundtrip = c.normalize(JSON.parse(JSON.stringify(cfg)));
    assert.equal(c.selectKeyboard(roundtrip,{id,vial:false}).standardLayout,p.id);
    const codes = new Set();
    for (const k of layout.keys) {
        assert.equal(k.codes.length,1); assert.equal(k.usages[0],k.codes[0]);
        assert(!codes.has(k.codes[0])); codes.add(k.codes[0]);
        assert(k.labels[0].length>0);
        assert(k.x>=0 && k.y>=0 && k.w>0 && k.h>0);
        assert(k.x+k.w<=layout.width && k.y+k.h<=layout.height);
    }
    for (let i=0;i<layout.keys.length;i++) for (const b of layout.keys.slice(i+1)) {
        const a=layout.keys[i];
        assert(a.x+a.w<=b.x || b.x+b.w<=a.x || a.y+a.h<=b.y || b.y+b.h<=a.y,`${p.id}: overlapping keys`);
    }
}
console.log('All selectable layouts: geometry, unique event codes and saved-profile round trips passed.');
