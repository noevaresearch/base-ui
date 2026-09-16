"""A/B control for the context-seam fix.

Reverts ONLY the `reactive_graph::owner::provide_context` qualification to the prelude
re-export, runs the two seam tests, then restores the file byte-for-byte and verifies the
restore. Proves the new host test fails on the broken provider rather than passing either
way.
"""
import io
import subprocess
import sys

PATH = 'crates/leptos-ui/src/toggle_group.rs'
GOOD = '    reactive_graph::owner::provide_context(runtime.context_value(value));\n'
BAD = '    provide_context(runtime.context_value(value));\n'

original = io.open(PATH, encoding='utf-8').read()
assert GOOD in original, 'the qualified call is not in the file — refusing to run'
broken = original.replace(GOOD, BAD, 1)
assert broken != original

io.open(PATH, 'w', encoding='utf-8').write(broken)
try:
    proc = subprocess.run(
        ['cargo', 'test', '-p', 'base-ui-leptos', '--lib', 'toggle_group'],
        capture_output=True, text=True, timeout=900,
        env={'CARGO_BUILD_JOBS': '2', 'PATH': '/data/.cargo/bin:/usr/bin:/bin', 'HOME': '/data'},
    )
    out = proc.stdout + proc.stderr
    for line in out.splitlines():
        if 'test result' in line or line.startswith('    toggle_group_tests::'):
            print('CONTROL:', line.strip())
    print('CONTROL exit code:', proc.returncode)
finally:
    io.open(PATH, 'w', encoding='utf-8').write(original)

restored = io.open(PATH, encoding='utf-8').read()
print('restored byte-identical:', restored == original)
if restored != original:
    sys.exit(2)
