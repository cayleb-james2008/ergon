local Http=game:GetService("HttpService")
assert(game:GetService("RunService"):IsStudio() and not game:GetService("RunService"):IsRunning() and game.PlaceId==0 and game.GameId==0 and workspace:GetAttribute("UseLiveDataStore")~=true,"local unpublished Edit required")
local function find(path) local node=game; for name in path:gmatch("[^/]+") do node=assert(node:FindFirstChild(name),"missing path "..path) end; return node end
local cfg=Http:JSONDecode("{\"session\": \"vinegar-ff754dbf4be4\", \"request\": \"900a08607ed14ba3b9a0cfdcfda77e17\", \"nonce\": \"17fcec9e904f45a0acce6a46670fee1b\", \"candidate\": \"1f24da33cbfd4b6f7c5b5cffb6c29bf954adc2129149fffcf57a066bba0c064c\", \"origin\": \"http://127.0.0.1:41463/17fcec9e904f45a0acce6a46670fee1b/\"}")
local before=Http.HttpEnabled;local after=before
local ok,result=xpcall(function()
local storage=game:GetService("ServerStorage")
assert(not storage:FindFirstChild("AtlasPreparation") and workspace:GetAttribute("AtlasNonce")==nil and workspace:GetAttribute("AtlasConfig")==nil,"foreign atlas preparation")
assert(not workspace:FindFirstChild("Nightshift"),"unrelated generated Edit world")
Http.HttpEnabled=true
local manifest=Http:JSONDecode(Http:GetAsync(cfg.origin.."manifest",true));assert(manifest.candidate==cfg.candidate,"candidate mismatch")
for _,row in manifest.sources do local node=find(row.parent.."/"..row.name);assert(node.ClassName==row.class and node.Source==Http:GetAsync(cfg.origin..row.file,true),"Source mismatch "..row.file) end
for _,row in manifest.helpers do assert(not find(row.parent):FindFirstChild(row.name),"helper occupied") end
local starters=Http:JSONDecode("{\"AtlasServerStarter\": [\"ServerScriptService/Nightshift\", \"Script\", \"local n=assert(workspace:GetAttribute(\\\"AtlasNonce\\\")); local ok,err=xpcall(function() local seed=workspace:GetAttribute(\\\"USGWorldSeed\\\"); if seed~=nil then local H0=game:GetService(\\\"HttpService\\\"); local t0=os.clock(); local World=require(assert(script.Parent:WaitForChild(\\\"World\\\",20))); local WorldSeed=require(assert(script.Parent:WaitForChild(\\\"AtlasWorldSeed\\\",20))); WorldSeed.apply(World,seed); local root=workspace:FindFirstChild(\\\"Nightshift\\\"); workspace:SetAttribute(\\\"AtlasStarterTrace\\\",H0:JSONEncode({t0=t0,applied=WorldSeed.applied(World),seed=seed,rootSeed=if root then root:GetAttribute(\\\"WorldSeed\\\") else nil})) end; local h=assert(script.Parent:WaitForChild(\\\"AtlasServer\\\",20)); require(h).start(n) end,debug.traceback); if not ok then local H=game:GetService(\\\"HttpService\\\"); local c=H:JSONDecode(workspace:GetAttribute(\\\"AtlasConfig\\\")); pcall(function() H:PostAsync(c.origin..\\\"report\\\",H:JSONEncode({session=c.session,request=c.request,nonce=n,phase=\\\"runtime\\\",ok=false,report={error=tostring(err),worldTrace=workspace:GetAttribute(\\\"AtlasWorldTrace\\\"),starter=workspace:GetAttribute(\\\"AtlasStarterTrace\\\")}}),Enum.HttpContentType.ApplicationJson,false) end); error(err) end\"], \"AtlasStarter\": [\"StarterPlayer/StarterPlayerScripts\", \"LocalScript\", \"local deadline=os.clock()+20; local n=workspace:GetAttribute(\\\"AtlasNonce\\\"); repeat task.wait(.05);n=workspace:GetAttribute(\\\"AtlasNonce\\\") until n or os.clock()>deadline; assert(n,\\\"atlas nonce missing\\\"); local h=assert(script.Parent:WaitForChild(\\\"AtlasClient\\\",20)); local m=require(h); local ok,err=xpcall(function() m.start(n) end,debug.traceback); if not ok then if m.abort then m.abort(err) else local e=game:GetService(\\\"ReplicatedStorage\\\"):WaitForChild(\\\"Nightshift\\\"):FindFirstChild(\\\"AtlasRelay\\\"); if e then e:FireServer({nonce=n,kind=\\\"error\\\",error=tostring(err)}) end end end\"]}")
for name,row in starters do assert(not find(row[1]):FindFirstChild(name),"starter occupied") end
local state=Instance.new("Folder");state.Name="AtlasPreparation";state:SetAttribute("AtlasOwned",cfg.nonce);state:SetAttribute("HttpBefore",before)
state:SetAttribute("FaresWasNil",workspace:GetAttribute("StudioStartFares")==nil);state:SetAttribute("FaresBefore",workspace:GetAttribute("StudioStartFares"));state.Parent=storage
workspace:SetAttribute("AtlasNonce",cfg.nonce);workspace:SetAttribute("AtlasConfig",Http:JSONEncode(cfg));workspace:SetAttribute("StudioStartFares",2)
workspace:SetAttribute("USGWorldSeed",2134783819)
for _,row in manifest.helpers do local node=Instance.new(row.class);node.Name=row.name;node.Source=Http:GetAsync(cfg.origin..row.file,true);node:SetAttribute("AtlasOwned",cfg.nonce);node:SetAttribute("SourceSHA256",manifest.sha256[row.file]);node.Parent=find(row.parent) end
for name,row in starters do local node=Instance.new(row[2]);node.Name=name;node.Source=row[3];node:SetAttribute("AtlasOwned",cfg.nonce);node.Parent=find(row[1]) end
after=true
return {candidate=cfg.candidate,verifiedSources=#manifest.sources,localPlace=game.PlaceId,skipOpeningFares=2,productionSourcesUntouched=true,namedSeed=2134783819}

end,debug.traceback)
local payload={session=cfg.session,request=cfg.request,nonce=cfg.nonce,phase="prepared",ok=ok}
if ok then payload.report=result else payload.report={error=tostring(result)} end
Http.HttpEnabled=true
local posted,err=pcall(Http.PostAsync,Http,cfg.origin.."report",Http:JSONEncode(payload),Enum.HttpContentType.ApplicationJson,false)
Http.HttpEnabled=after
assert(posted,err);assert(ok,result)
