local Http=game:GetService("HttpService")
assert(game:GetService("RunService"):IsStudio() and not game:GetService("RunService"):IsRunning() and game.PlaceId==0 and game.GameId==0 and workspace:GetAttribute("UseLiveDataStore")~=true,"local unpublished Edit required")
local function find(path) local node=game; for name in path:gmatch("[^/]+") do node=assert(node:FindFirstChild(name),"missing path "..path) end; return node end
local cfg=Http:JSONDecode("{\"session\": \"vinegar-ff754dbf4be4\", \"request\": \"900a08607ed14ba3b9a0cfdcfda77e17\", \"nonce\": \"17fcec9e904f45a0acce6a46670fee1b\", \"candidate\": \"1f24da33cbfd4b6f7c5b5cffb6c29bf954adc2129149fffcf57a066bba0c064c\", \"origin\": \"http://127.0.0.1:41463/17fcec9e904f45a0acce6a46670fee1b/\"}")
local before=Http.HttpEnabled;local after=before
local ok,result=xpcall(function()
local storage=game:GetService("ServerStorage");local state=storage:FindFirstChild("AtlasPreparation")
assert(state and state:GetAttribute("AtlasOwned")==cfg.nonce,"owned preparation history missing")
assert(type(state:GetAttribute("FaresWasNil"))=="boolean" and (state:GetAttribute("FaresWasNil") or state:GetAttribute("FaresBefore")~=nil),"preview history incomplete")
assert(workspace:GetAttribute("AtlasNonce")==nil or workspace:GetAttribute("AtlasNonce")==cfg.nonce,"foreign atlas nonce")
if state then after=state:GetAttribute("HttpBefore");assert(type(after)=="boolean","HTTP history missing") end
Http.HttpEnabled=true
local manifest=Http:JSONDecode(Http:GetAsync(cfg.origin.."manifest",true));assert(manifest.candidate==cfg.candidate,"candidate mismatch")
local starters=Http:JSONDecode("{\"AtlasServerStarter\": [\"ServerScriptService/Nightshift\", \"Script\", \"local n=assert(workspace:GetAttribute(\\\"AtlasNonce\\\")); local ok,err=xpcall(function() local seed=workspace:GetAttribute(\\\"USGWorldSeed\\\"); if seed~=nil then local H0=game:GetService(\\\"HttpService\\\"); local t0=os.clock(); local World=require(assert(script.Parent:WaitForChild(\\\"World\\\",20))); local WorldSeed=require(assert(script.Parent:WaitForChild(\\\"AtlasWorldSeed\\\",20))); WorldSeed.apply(World,seed); local root=workspace:FindFirstChild(\\\"Nightshift\\\"); workspace:SetAttribute(\\\"AtlasStarterTrace\\\",H0:JSONEncode({t0=t0,applied=WorldSeed.applied(World),seed=seed,rootSeed=if root then root:GetAttribute(\\\"WorldSeed\\\") else nil})) end; local h=assert(script.Parent:WaitForChild(\\\"AtlasServer\\\",20)); require(h).start(n) end,debug.traceback); if not ok then local H=game:GetService(\\\"HttpService\\\"); local c=H:JSONDecode(workspace:GetAttribute(\\\"AtlasConfig\\\")); pcall(function() H:PostAsync(c.origin..\\\"report\\\",H:JSONEncode({session=c.session,request=c.request,nonce=n,phase=\\\"runtime\\\",ok=false,report={error=tostring(err),worldTrace=workspace:GetAttribute(\\\"AtlasWorldTrace\\\"),starter=workspace:GetAttribute(\\\"AtlasStarterTrace\\\")}}),Enum.HttpContentType.ApplicationJson,false) end); error(err) end\"], \"AtlasStarter\": [\"StarterPlayer/StarterPlayerScripts\", \"LocalScript\", \"local deadline=os.clock()+20; local n=workspace:GetAttribute(\\\"AtlasNonce\\\"); repeat task.wait(.05);n=workspace:GetAttribute(\\\"AtlasNonce\\\") until n or os.clock()>deadline; assert(n,\\\"atlas nonce missing\\\"); local h=assert(script.Parent:WaitForChild(\\\"AtlasClient\\\",20)); local m=require(h); local ok,err=xpcall(function() m.start(n) end,debug.traceback); if not ok then if m.abort then m.abort(err) else local e=game:GetService(\\\"ReplicatedStorage\\\"):WaitForChild(\\\"Nightshift\\\"):FindFirstChild(\\\"AtlasRelay\\\"); if e then e:FireServer({nonce=n,kind=\\\"error\\\",error=tostring(err)}) end end end\"]}")
for _,row in manifest.helpers do local node=find(row.parent):FindFirstChild(row.name);if node then assert(node:GetAttribute("AtlasOwned")==cfg.nonce,"foreign helper");node:Destroy() end end
for name,row in starters do local node=find(row[1]):FindFirstChild(name);if node then assert(node:GetAttribute("AtlasOwned")==cfg.nonce,"foreign starter");node:Destroy() end end
local restored=true
if state then
  local prior=if state:GetAttribute("FaresWasNil") then nil else state:GetAttribute("FaresBefore")
  workspace:SetAttribute("StudioStartFares",prior);restored=workspace:GetAttribute("StudioStartFares")==prior;state:Destroy()
end
workspace:SetAttribute("AtlasNonce",nil);workspace:SetAttribute("AtlasConfig",nil)
workspace:SetAttribute("USGWorldSeed",nil)
for _,row in manifest.sources do local node=find(row.parent.."/"..row.name);assert(node.ClassName==row.class and node.Source==Http:GetAsync(cfg.origin..row.file,true),"final Source mismatch "..row.file) end
for _,row in manifest.helpers do assert(not find(row.parent):FindFirstChild(row.name),"helper remains") end
for name,row in starters do assert(not find(row[1]):FindFirstChild(name),"starter remains") end
local shared=find("ReplicatedStorage/Nightshift");assert(not shared:FindFirstChild("AtlasRelay") and not shared:FindFirstChild("AtlasPlan"),"runtime residue survived Stop")
Http.HttpEnabled=after
return {candidate=cfg.candidate,verifiedSources=#manifest.sources,cleaned=true,helpersAbsent=true,httpRestored=Http.HttpEnabled==after,
 skipOpeningRestored=restored,nonceAbsent=workspace:GetAttribute("AtlasNonce")==nil and workspace:GetAttribute("AtlasConfig")==nil,productionSourcesUntouched=true}

end,debug.traceback)
local payload={session=cfg.session,request=cfg.request,nonce=cfg.nonce,phase="cleanup",ok=ok}
if ok then payload.report=result else payload.report={error=tostring(result)} end
Http.HttpEnabled=true
local posted,err=pcall(Http.PostAsync,Http,cfg.origin.."report",Http:JSONEncode(payload),Enum.HttpContentType.ApplicationJson,false)
Http.HttpEnabled=after
assert(posted,err);assert(ok,result)
