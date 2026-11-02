#!/usr/bin/env python3
"""Text-level runtime probe of the page inside the phone-VM Chrome (no images, no screenshots).
Connects to the phone-VM DevTools endpoint (adb-forwarded), runs one Runtime.evaluate, prints JSON."""
import json, sys, urllib.request
import websocket

def target(port, needle):
    with urllib.request.urlopen(f'http://127.0.0.1:{port}/json/list', timeout=5) as r:
        targets = json.load(r)
    for t in targets:
        if needle in t.get('url', ''):
            return t
    raise SystemExit('no matching target')

def evaluate(ws_url, expression):
    ws = websocket.create_connection(ws_url, timeout=15, suppress_origin=True)
    try:
        ws.send(json.dumps({'id': 1, 'method': 'Runtime.evaluate',
                            'params': {'expression': expression, 'returnByValue': True, 'awaitPromise': True}}))
        while True:
            msg = json.loads(ws.recv())
            if msg.get('id') == 1:
                return msg
    finally:
        ws.close()

if __name__ == '__main__':
    port = int(sys.argv[1]); needle = sys.argv[2]; expr = sys.stdin.read()
    t = target(port, needle)
    out = evaluate(t['webSocketDebuggerUrl'], expr)
    print(json.dumps({'target': {'title': t.get('title'), 'url': t.get('url')}, 'result': out}, indent=1))
