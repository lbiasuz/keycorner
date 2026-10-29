"""Streams transient state to QML; never records typed input."""
import json
import os
from pathlib import Path
import select
import sys
import time
import tomllib
from core import devices, Vial, model, parse_report, infer_layer, label


def theme():
    for root in (Path.home()/'.local/state/omarchy/current/theme', Path.home()/'.config/omarchy/current/theme'):
        try:
            return tomllib.loads((root/'colors.toml').read_text())
        except (OSError, ValueError): pass
    return dict(background='#1a1b26', foreground='#c0caf5', accent='#7aa2f7',
                muted='#414868', lighter_background='#24283b', dark_foreground='#9aa5ce')


def emit(data):
    print(json.dumps(data, separators=(',',':')), flush=True)


def main():
    while True:
        fds = {}
        try:
            paths = devices()
            if 'vial' not in paths:
                raise OSError('Crosses 54 disconnected · reconnect to resume')
            vial = Vial(paths['vial'])
            try: layout = model(vial.snapshot())
            finally: vial.close()
            for kind in ('boot','shared'):
                if kind in paths:
                    fds[os.open(paths[kind], os.O_RDONLY | os.O_NONBLOCK)] = kind
            if not fds: raise OSError('No keyboard input interface available')
            emit(dict(type='layout', layout=layout, theme=theme()))
            held = {'keyboard':set(), 'media':set()}
            last_theme = 0
            while True:
                ready,_,_ = select.select(list(fds),[],[],0.5)
                changed = False
                for fd in ready:
                    while True:
                        try: report = os.read(fd,64)
                        except BlockingIOError: break
                        if not report: raise OSError('Keyboard disconnected')
                        parsed = parse_report(report, fds[fd])
                        if parsed:
                            group, codes = parsed
                            held[group] = codes; changed = True
                if changed:
                    pressed = held['keyboard'] | held['media']
                    emit(dict(type='state', pressed=sorted(pressed), estimate=infer_layer(layout,pressed),
                              output=' + '.join(label(k) for k in sorted(pressed))))
                if time.monotonic()-last_theme > 2:
                    emit(dict(type='theme', theme=theme())); last_theme=time.monotonic()
        except (OSError, ValueError, KeyError, TimeoutError) as exc:
            emit(dict(type='error', message=str(exc)))
        finally:
            for fd in fds: os.close(fd)
        time.sleep(2)


if __name__ == '__main__':
    try: main()
    except (KeyboardInterrupt, BrokenPipeError): sys.exit(0)
