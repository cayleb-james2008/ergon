#!/usr/bin/env python3
"""Read-only MCP probe: is Lighting.Technology readable from the Studio MCP execute_luau context?

No Play, no place change: one execute_luau in Edit mode, read-only.
"""
import json
import sys
import time
from pathlib import Path

ROOT = Path("/home/cayleb/Work/projects/untitled-street-game")
sys.path.insert(0, str(ROOT / "tools"))
from vinegar_lib import mcp, checks  # noqa: E402

CODE = r'''-- vinegar:diagnostic-technology-read (read-only)
local Http=game:GetService("HttpService")
local L=game:GetService("Lighting")
local out={}
local ok,res=pcall(function() return L.Technology end)
out.directOk=ok
out.directValue=if ok then tostring(res) else nil
out.directError=if ok then nil else tostring(res)
local okName,resName=pcall(function() return L.Technology.Name end)
out.nameOk=okName
out.nameValue=if okName then tostring(resName) else nil
out.nameError=if okName then nil else tostring(resName)
local okType,resType=pcall(function() return typeof(L.Technology) end)
out.typeOk=okType
out.typeValue=if okType then tostring(resType) else nil
out.studioMode=game:GetService("RunService"):IsStudio()
return Http:JSONEncode(out)'''


def main():
    result = {"ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "phase": "start"}
    cmd, args, env = mcp.server_command(ROOT)
    result["command"] = cmd
    session = mcp.StdioSession(cmd, *args, env=env) if env else mcp.StdioSession(cmd, *args)
    try:
        session.start()
        studio = None
        deadline = time.monotonic() + 30
        while True:
            raw = session.call("list_roblox_studios", {}, 60)
            studios = json.loads(mcp.text(raw) or "{}").get("studios") or []
            if studios:
                studio = studios[0]
                break
            if time.monotonic() > deadline:
                result.update(error="no studio listed within 30 s", raw=mcp.text(raw)[:300])
                break
            time.sleep(1)
        result["studio"] = studio
        if studio:
            raw = session.call("execute_luau", {"studio_id": studio["id"], "datamodel_type": "Edit", "code": CODE}, 120)
            result["execute"] = {"isError": raw.get("isError"), "text": mcp.text(raw)[:2000]}
    finally:
        result["close"] = session.close()
        print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
