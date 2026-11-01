#!/usr/bin/python3
"""CHECKER probe for the scratch fleet MCP facade: admin-socket token, then my own
JSON-RPC initialize / notifications/initialized / tools/list over HTTP, plus an
auth.refuse check. The bearer token is handled in memory only and is NEVER printed.
"""
import json
import socket
import sys
import time
import urllib.error
import urllib.request


def admin_op(sock_path, body):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(10)
    s.connect(sock_path)
    s.sendall((json.dumps(body) + "\n").encode())
    buf = b""
    while b"\n" not in buf:
        d = s.recv(65536)
        if not d:
            break
        buf += d
    s.close()
    return json.loads(buf.split(b"\n")[0])


def post(url, token, body, session=None, timeout=30):
    data = json.dumps(body).encode()
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
        "Authorization": "Bearer " + token,
    }
    if session:
        headers["Mcp-Session-Id"] = session
    req = urllib.request.Request(url, data=data, headers=headers, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.headers, resp.read()
    except urllib.error.HTTPError as exc:
        return exc.code, exc.headers, exc.read()
    except urllib.error.URLError as exc:
        raise SystemExit(f"BLOCKED mcp-unreachable: {exc}")


def rpc_payload(status, headers, raw):
    ctype = (headers.get("Content-Type") or "").lower()
    if "text/event-stream" in ctype:
        payloads = []
        for line in raw.decode("utf-8", "replace").splitlines():
            if line.startswith("data:"):
                payloads.append(line[5:].strip())
        return json.loads(payloads[-1]) if payloads else None
    if not raw.strip():
        return None
    return json.loads(raw.decode("utf-8"))


def hdr(headers, name):
    for key, value in headers.items():
        if key.lower() == name.lower():
            return value
    return None


def main():
    admin_sock, url = sys.argv[1], sys.argv[2]
    fails = []

    def check(ok, line):
        print(("PASS  " if ok else "FAIL  ") + line)
        if not ok:
            fails.append(line)

    # token over the admin socket (SO_PEERCRED peer uid 1000)
    try:
        answer = admin_op(admin_sock, {"op": "token"})
        token = (answer.get("result") or {}).get("token")
        check(bool(answer.get("ok")) and isinstance(token, str) and len(token) >= 32,
              f"admin-socket token op ok (len={len(token) if isinstance(token, str) else 0}; value never printed)")
    except OSError as exc:
        print(f"BLOCKED mcp-admin-socket: {exc}")
        return 3

    # wait for the HTTP listener
    deadline = time.time() + 60
    while time.time() < deadline:
        try:
            socket.create_connection(("127.0.0.1", int(url.rsplit(":", 1)[1].split("/")[0])), timeout=1).close()
            break
        except OSError:
            time.sleep(0.5)
    else:
        print("BLOCKED mcp-listener: not accepting after 60s")
        return 3

    init = {
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                   "clientInfo": {"name": "checker-install", "version": "0.0.0"}},
    }
    status, headers, raw = post(url, token, init)
    obj = rpc_payload(status, headers, raw)
    session = hdr(headers, "Mcp-Session-Id")
    ok = (status == 200 and isinstance(obj, dict) and obj.get("jsonrpc") == "2.0"
          and isinstance(obj.get("result"), dict)
          and obj["result"].get("protocolVersion") in ("2025-03-26", "2024-11-05")
          and isinstance(obj["result"].get("serverInfo"), dict))
    si = (obj or {}).get("result", {}).get("serverInfo", {}) if isinstance(obj, dict) else {}
    check(ok, f"initialize HTTP {status} protocol={obj.get('result', {}).get('protocolVersion') if isinstance(obj, dict) else None} "
              f"server={si.get('name')} {si.get('version')} session={'issued' if session else 'absent'}")

    status2, _h2, raw2 = post(url, token, {"jsonrpc": "2.0", "method": "notifications/initialized"}, session)
    check(status2 == 202, f"notifications/initialized HTTP {status2}")

    status3, headers3, raw3 = post(url, token, {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}, session)
    obj3 = rpc_payload(status3, headers3, raw3)
    tools = ((obj3 or {}).get("result") or {}).get("tools") if isinstance(obj3, dict) else None
    names = [t.get("name") for t in (tools or []) if isinstance(t, dict)]
    check(status3 == 200 and isinstance(tools, list) and len(names) > 0,
          f"tools/list HTTP {status3} {len(names)} tools: {', '.join(names)}")
    snapshot = " ".join(names)
    for required in ("fleet.status", "session.create", "board.op", "actions.status"):
        check(required in snapshot, f"tools/list contains {required}")

    wrong = "0" * 64
    status4, _h4, _r4 = post(url, wrong, {"jsonrpc": "2.0", "id": 3, "method": "tools/list", "params": {}})
    check(status4 == 401, f"auth.refuse wrong bearer HTTP {status4}")

    print(f"mcp-probe: {'PASS' if not fails else 'FAIL'} — {len(fails)} failed")
    return 0 if not fails else 1


raise SystemExit(main())
