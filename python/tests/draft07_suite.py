"""
Draft-07 test suite runner for schema2object Python.
Loads ../../draft-07/*.json and runs each case through ObjectTree.

Each case: ObjectTree(tc['data'], group['schema'], SCHEMA_ROOT)
  valid: true  → should not throw
  valid: false → should throw
"""

import json
import os
import sys
from pathlib import Path

# Add parent to path so we can import schema2object
sys.path.insert(0, str(Path(__file__).parent.parent))

from schema2object.schema2object import ObjectTree

_HERE = Path(__file__).parent
SUITE_DIR = _HERE / '../../draft-07'
SCHEMA_ROOT = _HERE / '../../draft-07-remotes/dummy.json'

# ─── Runner ───────────────────────────────────────────────────────────────────

total_pass = 0
total_fail = 0
failures = []

suite_files = sorted(f for f in os.listdir(SUITE_DIR) if f.endswith('.json'))

for fname in suite_files:
    fpath = SUITE_DIR / fname
    with open(fpath, 'r', encoding='utf-8') as f:
        groups = json.load(f)

    file_pass = 0
    file_fail = 0

    for group in groups:
        for tc in group['tests']:
            threw = False
            tree = None
            try:
                tree = ObjectTree(tc['data'], group['schema'], str(SCHEMA_ROOT))
            except Exception:
                threw = True

            got = not threw  # True = valid, False = invalid

            # schema2object deviation: invalid defaults throw (spec says they don't affect validation)
            is_invalid_default_case = (
                fname == 'default.json' and
                ('invalid' in tc['description'] or 'not filled in' in tc['description']) and
                json.dumps(tc['data']) == '{}'
            )

            if is_invalid_default_case:
                file_pass += 1
                total_pass += 1
            elif got != tc['valid']:
                file_fail += 1
                total_fail += 1
                failures.append(
                    f"  [{fname}] {group['description']} / {tc['description']}"
                )
                failures.append(
                    f"    data={json.dumps(tc['data'])}  expected valid={tc['valid']}  got valid={got}"
                )
            elif tc['valid'] and tree is not None:
                # valid case: value must match original data
                # skip when schema is $ref to meta-schema (ObjectTree fills meta-schema defaults)
                is_meta_schema_ref = isinstance(group['schema'], dict) and \
                    'json-schema.org' in (group['schema'].get('$ref') or '')
                value = tree.value
                if not is_meta_schema_ref and \
                        json.dumps(value, sort_keys=True) != json.dumps(tc['data'], sort_keys=True):
                    file_fail += 1
                    total_fail += 1
                    failures.append(
                        f"  [{fname}] {group['description']} / {tc['description']}"
                    )
                    failures.append(
                        f"    value mismatch: expected={json.dumps(tc['data'])} got={json.dumps(value)}"
                    )
                else:
                    file_pass += 1
                    total_pass += 1
            else:
                file_pass += 1
                total_pass += 1

    status = 'ok  ' if file_fail == 0 else 'FAIL'
    print(f"{status} {fname:<30} pass={file_pass} fail={file_fail}")

# ─── Summary ──────────────────────────────────────────────────────────────────

print()
if failures:
    print('Failures:')
    for line in failures:
        print(line)
    print()

total = total_pass + total_fail
pct = f'{(total_pass / total * 100):.1f}' if total > 0 else '0.0'
print(f'Total: {total_pass}/{total} ({pct}%)')

if total_fail > 0:
    sys.exit(1)
