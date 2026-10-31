import sys
from pathlib import Path
import unittest
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'reference/python/keycorner'))
from standard import EVENT, Reader, layout
from keyboard_devices import keyboard_capable, token
from permissions import make_rule, destination


def event(kind, code, value=0): return EVENT.pack(0, 0, kind, code, value)


class StandardTests(unittest.TestCase):
    def test_presets_have_expected_unique_keys_and_valid_geometry(self):
        for preset, count in [('ansi104',104),('ansi87',87),('ansi60',61)]:
            m=layout(preset)
            self.assertEqual(len(m['keys']),count)
            self.assertEqual(len({k['usages'][0] for k in m['keys']}),count)
            self.assertEqual(m['layers'],1)
            for k in m['keys']:
                self.assertTrue(0 <= k['x'] < k['x']+k['w'] <= m['width'])
                self.assertTrue(0 <= k['y'] < k['y']+k['h'] <= m['height'])
        with self.assertRaises(ValueError): layout('../../bad')

    def test_partial_events_repeat_and_release(self):
        with patch('standard.key_state', return_value=set()): r=Reader(1)
        data=event(1,30,1)+event(0,0)
        self.assertEqual(r.feed(data[:9]),[])
        self.assertEqual(r.feed(data[9:]),[{30}])
        self.assertEqual(r.feed(event(1,30,2)+event(0,0)),[{30}])
        self.assertEqual(r.feed(event(1,30,0)+event(0,0)),[set()])

    def test_overflow_discards_events_until_resynchronization(self):
        with patch('standard.key_state', return_value={30}): r=Reader(1)
        with patch('standard.key_state', return_value={42}) as snapshot:
            self.assertEqual(r.feed(event(0,3)+event(1,40,1)+event(0,0)),[set(),{42}])
            snapshot.assert_called_once_with(1)
        self.assertEqual(r.held,{42})

    def test_capabilities_exclude_mouse_and_consumer_only_interfaces(self):
        value=sum(1<<k for k in (28,30,44,42))
        self.assertTrue(keyboard_capable(hex(value)[2:]))
        self.assertFalse(keyboard_capable('0'))
        self.assertNotEqual(token('keyboard1'),token('keyboard2'))


class PermissionTests(unittest.TestCase):
    def setUp(self):
        self.device={'id':'a'*24,'match':{'usb_port':'1-6','vendor':'3034','product':'5659','serial':'vial:f64c2b3c'}}

    def test_standard_and_vial_are_separate_and_narrowly_scoped(self):
        standard=make_rule(self.device,'standard');vial=make_rule(self.device,'vial')
        self.assertIn('ENV{ID_INPUT_KEYBOARD}=="1"',standard)
        self.assertIn('KERNELS=="1-6"',standard)
        self.assertNotIn('hidraw',standard)
        self.assertIn('SUBSYSTEM=="hidraw"',vial)
        for rule in (standard,vial):
            self.assertNotIn('GROUP=',rule);self.assertNotIn('MODE=',rule);self.assertNotIn('RUN',rule)
        self.assertNotEqual(destination(self.device,'standard'),destination(self.device,'vial'))

    def test_reject_rule_injection_patterns_and_path_traversal(self):
        for bad in ['bad" , RUN+="command','*','a?b','a[b]','a\\b','x\ny','x%k','$env{HOME}']:
            self.device['match']['serial']=bad
            with self.subTest(bad=bad),self.assertRaises(ValueError):make_rule(self.device,'standard')
        self.device['id']='../../escape'
        with self.assertRaises(ValueError):destination(self.device,'standard')

    def test_internal_keyboard_rule_requires_stable_identity(self):
        self.device['match']={'name':'AT Translated Set 2 keyboard','phys':'isa0060/serio0/input0','uniq':''}
        rule=make_rule(self.device,'standard')
        self.assertIn('ATTRS{phys}=="isa0060/serio0/input0"',rule)
        with self.assertRaises(ValueError):make_rule(self.device,'vial')
        self.device['match']['phys']=''
        with self.assertRaises(ValueError):make_rule(self.device,'standard')


class StandardBridgeTests(unittest.TestCase):
    def test_standard_bridge_never_queries_vial_and_closes_on_parent_eof(self):
        import bridge
        import os
        from types import SimpleNamespace
        from contextlib import ExitStack
        messages=[]
        packet=event(1,30,1)+event(0,0)+event(1,30,0)+event(0,0)
        pending=[packet]
        def read(fd, size):
            if fd==10:return b''
            if pending:return pending.pop(0)
            raise BlockingIOError()
        with ExitStack() as stack:
            def mocked(name, **kwargs):return stack.enter_context(patch(name,**kwargs))
            mocked('bridge.sys.stdin',new=SimpleNamespace(fileno=lambda:10))
            mocked('bridge.resource.setrlimit')
            mocked('bridge.os.set_blocking')
            opened=mocked('bridge.os.open',return_value=20)
            closed=mocked('bridge.os.close')
            mocked('bridge.os.fstat',return_value=SimpleNamespace(st_dev=1,st_ino=2,st_rdev=3))
            mocked('bridge.os.read',side_effect=read)
            mocked('bridge.select.select',side_effect=[([20],[],[]),([10],[],[])])
            mocked('bridge.selected_device',return_value={'events':['/dev/input/event99'],'eventIdentity':{'/dev/input/event99':[1,2,3]}})
            mocked('standard.key_state',return_value=set())
            vial=mocked('bridge.Vial',side_effect=AssertionError('Vial must not be used'))
            mocked('bridge.emit',side_effect=messages.append)
            bridge.main('a'*24,'standard','ansi87')
            self.assertEqual(opened.call_args.args[1] & os.O_ACCMODE,os.O_RDONLY)
            closed.assert_called_once_with(20)
            vial.assert_not_called()
        self.assertEqual(messages[0]['layout']['source'],'standard')
        self.assertEqual([m['pressed'] for m in messages if m['type']=='state'],[[],[30],[]])

    def test_permission_install_requires_elevation(self):
        from permissions import root_install
        with patch('permissions.os.geteuid',return_value=1000):
            with self.assertRaises(PermissionError):root_install({},'standard')

    def test_root_helper_writes_only_scoped_rule_and_triggers_selected_nodes(self):
        import tempfile
        import permissions
        from pathlib import Path
        device={'id':'a'*24,'match':{'usb_port':'1-6','vendor':'3034','product':'5659','serial':'vial:f64c2b3c'},'nodes':['/dev/input/event3']}
        with tempfile.TemporaryDirectory() as tmp, patch.object(permissions,'RULE_DIR',Path(tmp)), patch('permissions.os.geteuid',return_value=0), patch('permissions.subprocess.run') as run:
            installed=Path(permissions.root_install(device,'standard'))
            self.assertEqual(installed.parent,Path(tmp))
            self.assertEqual(installed.read_text(),permissions.make_rule(device,'standard'))
            self.assertEqual(installed.stat().st_mode & 0o777,0o644)
            self.assertEqual(len(list(Path(tmp).iterdir())),1)
            self.assertEqual(run.call_args_list[-1].args[0],['/usr/bin/udevadm','trigger','--settle','--action=change','/dev/input/event3'])
