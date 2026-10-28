#!/usr/bin/env python3
"""Drive the real stdio proxy against a native capture window; assert receipts."""
import json
import pathlib
import subprocess
import sys
import uuid

root = pathlib.Path(__file__).resolve().parents[2]
process = subprocess.Popen([str(root / 'target/debug/quantum-mcp'), '--connect', sys.argv[1]], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
serial = 0

def rpc(method, params):
    global serial
    serial += 1
    process.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': serial, 'method': method, 'params': params}) + '\n')
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    assert response['id'] == serial and 'error' not in response, response
    return response['result']

def tool(name, args):
    result = rpc('tools/call', {'name': 'quantum_' + name, 'arguments': args})
    assert not result.get('isError'), result
    return result['structuredContent']

rpc('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'capture-agent', 'version': '1'}})
process.stdin.write(json.dumps({'jsonrpc': '2.0', 'method': 'notifications/initialized'}) + '\n')
process.stdin.flush()
cap = tool('capabilities_get', {})
doc = cap['data']['documents'][0]
model = doc['model_id']
revision = doc['revision']
epoch = cap['epoch']
receipts = []

def edit(tool_name, **args):
    global revision
    data = tool(tool_name, dict(model_id=model, expected_epoch=epoch, expected_revision=revision, operation_id=str(uuid.uuid4()), **args))['data']
    revision = data['revision']
    receipts.append({"tool": tool_name, 'revision': revision, 'added': data.get('added', 0), 'recolored': data.get('recolored', 0)})
    return data

if len(sys.argv) < 3 or sys.argv[2] == 'seed':
    fixture = json.loads((root / 'examples/autumn-workshop.voxl').read_text())
    detail = fixture['layers'][-1]
    edit('versions_save', name='Before the detail pass')
    anchor = max(fixture['layers'][2]['voxels'], key=lambda v: v[1])[:3]
    thread = edit('comments_add', anchor={'kind': 'voxel', 'at': anchor}, text='The autumn crown reads well against the slate roof. Keep this warm highlight.', layer=fixture['layers'][2]['id'])
    edit('comments_reply', thread_id=thread['thread_id'], text='Detail pass complete. Ready for a silhouette review.')
    edit('versions_save', name='Review checkpoint')
    cells = detail['voxels'][:6]
    edit('voxels_set', layer=detail['id'], voxels=[{'at': v[:3], 'slot': 17 if v[3] != 17 else 18} for v in cells], label='Warm the window details')
    edit('voxels_box', layer=detail['id'], region={'min': [11, 4, 17], 'max': [14, 5, 18]}, slot=17, label='Add an amber garden detail')
    tool('render', {'model_id': model, 'revision': revision, 'views': ['front', 'right', 'back'], 'width': 160, 'height': 160})
    assert int(revision) >= 6
    assert any(r['recolored'] > 0 for r in receipts)
    assert any(r['added'] > 0 for r in receipts)
elif sys.argv[2] == 'verify':
    comments = tool('comments_list', {'model_id': model})['data']['comments']
    messages = [m for c in comments for m in c['thread']['messages']]
    assert any(m['origin'] == 'human' and m['text'] == 'The warm highlights look balanced.' for m in messages), messages
    versions = tool('versions_list', {'model_id': model})['data']['versions']
    assert any(v['origin'] == 'human' and v['name'] == 'Human review' for v in versions), versions
    assert revision == '8', revision  # Preview must not commit or trigger editing shortcuts.
print(json.dumps({'revision': revision, 'receipts': receipts}))
process.stdin.close()
assert process.wait(timeout=10) == 0
