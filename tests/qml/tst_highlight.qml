import QtQuick
import QtTest
import '../../Highlight.js' as Highlight
TestCase {
    name: 'Highlight'
    function key(code, usage) { return {codes:[code],usages:[usage]} }
    function test_shifted_assignment_wins() {
        var keys = [key(32,32),key(0x0220,32)] // 3 and #
        compare(Highlight.keys(keys,0,null,[225,32],'vial'),[false,true])
        compare(Highlight.keys(keys,0,null,[32],'vial'),[true,false])
        compare(Highlight.keys(keys,0,null,[],'vial'),[false,false])
    }
    function test_live_and_transparent() {
        var keys = [key(32,32),key(0,0)]
        var live = [{code:32,usage:32},{code:0x0220,usage:32}]
        compare(Highlight.keys(keys,0,live,[225,32],'vial'),[false,true])
        compare(Highlight.keys([{codes:[0x0220,1],usages:[32,32]}],1,null,[32],'vial'),[false])
    }
    function test_plain_shift_and_modtap() {
        compare(Highlight.keys([key(32,32)],0,null,[225,32],'vial'),[true])
        compare(Highlight.keys([key(0x2804,4)],0,null,[4],'vial'),[true])
    }
    function test_modifier_sides_and_specificity() {
        var keys = [key(4,4),key(0x0204,4),key(0x0304,4),key(0x1204,4)]
        compare(Highlight.keys(keys,0,null,[224,225,4],'vial'),[false,false,true,false])
        compare(Highlight.keys(keys,0,null,[229,4],'vial'),[false,false,false,true])
    }
    function test_standard_unchanged() {
        compare(Highlight.keys([key(0x0220,32)],0,null,[32],'standard'),[true])
    }
    function test_active_modifier_holds() {
        compare(Highlight.modifiers([4,227,229],'vial').map(function(m) { return m.label }),['Super','RShift'])
        compare(Highlight.modifiers([42,97,125],'standard').map(function(m) { return m.label }),['Shift','Super','RCtrl'])
        compare(Highlight.modifiers([4,32],'vial'),[])
        compare(Highlight.modifiers([],'vial'),[])
    }
}
