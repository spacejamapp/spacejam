#!/usr/bin/env python3

import json
from pathlib import Path

# dhat-heap.json is in project root, script may be run from root or scripts/
heap_path = Path(__file__).resolve().parent.parent / 'dhat-heap.json'
with open(heap_path) as f:
    data = json.load(f)

ftbl = data['ftbl']
pps = data['pps']

# Sum up total bytes and bytes-at-end by caller function
from collections import defaultdict

# Group by full backtrace but show the first app-level frame
tb_by_func = defaultdict(lambda: {'tb': 0, 'tbk': 0, 'eb': 0, 'ebk': 0, 'gb': 0, 'mb': 0})
for p in pps:
    frames = [ftbl[i] for i in p['fs'] if i < len(ftbl)]
    # Find the deepest app frame (skip alloc/dhat/core internals)
    app_frames = []
    for f in frames:
        if 'alloc::' not in f and 'dhat::' not in f and '__rustc' not in f:
            app_frames.append(f.split(': ', 1)[-1].split(' (')[0] if ': ' in f else f)
    key = ' -> '.join(app_frames[:3]) if app_frames else 'unknown'
    d = tb_by_func[key]
    d['tb'] += p.get('tb', 0)
    d['tbk'] += p.get('tbk', 0)
    d['eb'] += p.get('eb', 0)
    d['ebk'] += p.get('ebk', 0)
    d['gb'] += p.get('gb', 0)
    d['mb'] = max(d['mb'], p.get('mb', 0))

print('=== TOP 15 call paths by TOTAL BYTES ===')
for key, d in sorted(tb_by_func.items(), key=lambda x: x[1]['tb'], reverse=True)[:15]:
    tb, eb, gb = d['tb'], d['eb'], d['gb']
    print(f'  tb={tb/1e6:>10.1f}MB  eb={eb/1e6:>8.1f}MB  gb={gb/1e6:>8.1f}MB  | {key}')

print()
print('=== TOP 15 call paths by BYTES AT END (leaks) ===')
for key, d in sorted(tb_by_func.items(), key=lambda x: x[1]['eb'], reverse=True)[:15]:
    if d['eb'] == 0: break
    eb, tb, ebk = d['eb'], d['tb'], d['ebk']
    print(f'  eb={eb/1e6:>8.1f}MB  tb={tb/1e6:>10.1f}MB  ebk={ebk:>6}  | {key}')

print()
print('=== TOP 15 call paths by BYTES AT GLOBAL MAX (peak) ===')
for key, d in sorted(tb_by_func.items(), key=lambda x: x[1]['gb'], reverse=True)[:15]:
    if d['gb'] == 0: break
    gb, tb = d['gb'], d['tb']
    print(f'  gb={gb/1e6:>8.1f}MB  tb={tb/1e6:>10.1f}MB  | {key}')

# Now show specifically pvmi breakdown
print()
print('=== pvmi::Interpreter::state breakdown ===')
pvmi_points = []
for p in pps:
    frames = [ftbl[i] for i in p['fs'] if i < len(ftbl)]
    frame_str = ' | '.join(frames)
    if 'pvmi::Interpreter::state' in frame_str:
        pvmi_points.append((p, frames))
pvmi_points.sort(key=lambda x: x[0].get('tb', 0), reverse=True)
for p, frames in pvmi_points[:10]:
    clean = [f.split(': ', 1)[-1].split(' (')[0] if ': ' in f else f for f in frames if 'alloc::' not in f and 'dhat::' not in f]
    tb, mb, gb = p['tb'], p['mb'], p['gb']
    print(f'  tb={tb/1e6:>10.1f}MB  mb={mb/1e6:>6.1f}MB  gb={gb/1e6:>6.1f}MB  | {" -> ".join(clean[:5])}')
