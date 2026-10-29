"""Event-driven keyboard state stream. No typing log, network or periodic polling."""
import json
import lzma
import os
from pathlib import Path
import resource
import select
import sys
import tomllib
from core import devices, Vial, model, parse_report, infer_layer


def theme():
    for root in (Path.home()/'.local/state/omarchy/current/theme', Path.home()/'.config/omarchy/current/theme'):
        try:
            with (root/'colors.toml').open('rb') as stream:
                raw = stream.read(65537)
            if len(raw) > 65536:
                continue
            return tomllib.loads(raw.decode())
        except (OSError, ValueError): pass
    return dict(background='#1a1b26', foreground='#c0caf5', accent='#7aa2f7',
                muted='#414868', lighter_background='#24283b', dark_foreground='#9aa5ce')


def emit(data):
    print(json.dumps(data, separators=(',',':')), flush=True)


def state_message(layout, held, suggestions=True):
    pressed = held['keyboard'] | held['media']
    return dict(type='state', pressed=sorted(pressed),
                estimate=infer_layer(layout, pressed) if suggestions else None)


def main():
    # Keyboard state must not be saved in a Python core dump on failure.
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    control_fd = sys.stdin.fileno()
    os.set_blocking(control_fd, False)
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
            current_theme = theme()
            emit(dict(type='layout', layout=layout, theme=current_theme))
            held = {'keyboard':set(), 'media':set()}
            previous = None
            suggestions = True
            control_buffer = b''
            while True:
                ready,_,_ = select.select([*fds, control_fd], [], [])
                changed = False
                if control_fd in ready:
                    commands = os.read(control_fd, 4096)
                    if not commands: return  # Parent closed the pipe: stop capture.
                    control_buffer += commands
                    if len(control_buffer) > 8192:
                        raise ValueError('Oversized control message')
                    while b'\n' in control_buffer:
                        command,control_buffer = control_buffer.split(b'\n',1)
                        if command == b'theme':
                            updated = theme()
                            if updated != current_theme:
                                current_theme = updated
                                emit(dict(type='theme', theme=current_theme))
                        elif command in (b'suggest:0', b'suggest:1'):
                            suggestions = command == b'suggest:1'; changed = True
                for fd in ready:
                    if fd == control_fd: continue
                    # Bound each batch so a noisy device cannot starve control messages.
                    for _ in range(256):
                        try: report = os.read(fd,64)
                        except BlockingIOError: break
                        if not report: raise OSError('Keyboard disconnected')
                        parsed = parse_report(report, fds[fd])
                        if parsed:
                            group, codes = parsed
                            if held[group] != codes:
                                held[group] = codes; changed = True
                if changed:
                    message = state_message(layout, held, suggestions)
                    if message != previous:
                        emit(message); previous = message
        except (OSError, ValueError, KeyError, IndexError, TypeError, lzma.LZMAError) as exc:
            emit(dict(type='error', message=str(exc)))
        finally:
            for fd in fds: os.close(fd)
        # Disconnected devices are rescanned every two seconds; connected devices
        # block on input. Respect parent shutdown even during a reconnect wait.
        if select.select([control_fd], [], [], 2)[0]:
            if not os.read(control_fd, 4096): return


if __name__ == '__main__':
    try:
        if sys.argv[1:] == ['--theme-only']: emit(dict(theme=theme()))
        else: main()
    except (KeyboardInterrupt, BrokenPipeError): sys.exit(0)
