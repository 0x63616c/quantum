#!/usr/bin/env python3
"""Record a visible three-voxel garden edit through real MCP and capture its detail."""
import base64
import hashlib
import json
from pathlib import Path
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'screenshots/site/proof'
OUTPUT.mkdir(parents=True, exist_ok=True)
process = subprocess.Popen([str(ROOT / 'target/debug/voxelite-mcp'), '--root', str(ROOT)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
sequence = 0

def rpc(method, params):
    global sequence
    sequence += 1
    process.stdin.write(json.dumps(dict(jsonrpc='2.0', id=sequence, method=method, params=params)) + '\n')
    process.stdin.flush()
    while True:
        result = json.loads(process.stdout.readline())
        if result.get('id') == sequence:
            assert 'error' not in result, result
            return result['result']

def call(name, args):
    result = rpc('tools/call', dict(name='voxelite_' + name, arguments=args))
    assert not result.get('isError'), result
    return result

try:
    rpc('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='site-proof', version='1')))
    process.stdin.write(json.dumps(dict(jsonrpc='2.0', method='notifications/initialized')) + '\n'); process.stdin.flush()
    epoch = call('capabilities_get', {})['structuredContent']['epoch']
    opened = call('document_open', dict(path='examples/autumn-workshop.voxl', expected_epoch=epoch, operation_id=str(uuid.uuid4())))['structuredContent']['data']
    model = opened['model_id']
    fixture = json.loads((ROOT / 'examples/autumn-workshop.voxl').read_text())
    layer = fixture['layers'][-1]['id']
    camera = dict(eye=[26, 22, 35], target=[12.5, 8, 17.5], up=[0, 1, 0], projection='orthographic', ortho_height=9)
    evidence = dict(source='examples/autumn-workshop.voxl', operation=dict(tool='voxelite_voxels_box', region=dict(min=[11,8,17], max=[14,9,18]), slot=21), source_sha256=hashlib.sha256((ROOT / 'examples/autumn-workshop.voxl').read_bytes()).hexdigest(), camera=camera, captures=[])
    for state in ['before', 'after']:
        revision = call('document_info', dict(model_id=model))['structuredContent']['data']['revision']
        if state == 'after':
            result = call('voxels_box', dict(model_id=model, expected_epoch=epoch, expected_revision=revision, operation_id=str(uuid.uuid4()), layer=layer, region=dict(min=[11, 8, 17], max=[14, 9, 18]), slot=21, label='Add an amber garden detail'))['structuredContent']['data']
            assert result['added'] == 3, result
            evidence['edit'] = {key: result[key] for key in ['added', 'removed', 'recolored', 'revision']}
            revision = result['revision']
        for theme, background in [('dark', [0, 0, 0, 255]), ('light', [255, 255, 255, 255])]:
            result = call('render', dict(model_id=model, revision=revision, camera=camera, width=512, height=384, background=background, image_mode='individual'))
            image = next(block for block in result['content'] if block['type'] == 'image')
            data = base64.b64decode(image['data'])
            assert data.startswith(b'\x89PNG\r\n\x1a\n')
            name = f'{state}-{theme}.png'
            (OUTPUT / name).write_bytes(data)
            (ROOT / 'site/assets' / name).write_bytes(data)
            evidence['captures'].append(dict(file=name, revision=revision, sha256=hashlib.sha256(data).hexdigest()))
    for theme in ('dark', 'light'):
        assert (OUTPUT / f'before-{theme}.png').read_bytes() != (OUTPUT / f'after-{theme}.png').read_bytes(), 'Edit must be visible'
    manifest = json.dumps(evidence, indent=2) + '\n'
    (OUTPUT / 'provenance.json').write_text(manifest)
    (ROOT / 'site/assets/proof-provenance.json').write_text(manifest)
    print('Captured four actual MCP detail renders; verified exactly three added voxels.')
finally:
    process.stdin.close()
    process.wait(timeout=10)
