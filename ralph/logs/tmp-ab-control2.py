"""Repeat the A/B control, this time naming exactly which tests fail without the fix."""
import io
import subprocess
import sys

PATH = 'crates/leptos-ui/src/toggle_group.rs'
GOOD = '    reactive_graph::owner::provide_context(runtime.context_value(value));\n'
BAD = '    provide_context(runtime.context_value(value));\n'

original = io.open(PATH, encoding='utf-8').read()
assert GOOD in original and BAD not in original
io.open(PATH, 'w', encoding='utf-8').write(original.replace(GOOD, BAD, 1))
try:
    proc = subprocess.run(
        ['cargo', 'test', '-p', 'base-ui-leptos', '--lib', 'toggle_group'],
        capture_output=True, text=True, timeout=900,
        env={'CARGO_BUILD_JOBS': '2', 'PATH': '/data/.cargo/bin:/usr/bin:/bin', 'HOME': '/data'},
    )
    io.open('/tmp/tg-control.log', 'w', encoding='utf-8').write(proc.stdout + proc.stderr)
    print('exit:', proc.returncode)
    text = proc.stdout + proc.stderr
    for line in text.splitlines():
        if line.startswith('test toggle_group') or 'test result' in line:
            print(line.strip())
    print('--- failing names ---')
    grab = False
    for line in text.splitlines():
        if line.strip() == 'failures:':
            grab = True
            continue
        if grab and line.startswith('    ') and '::' in line:
            print(line.strip())
        elif grab and line.strip() == '':
            grab = False
finally:
    io.open(PATH, 'w', encoding='utf-8').write(original)

print('restored byte-identical:', io.open(PATH, encoding='utf-8').read() == original)
