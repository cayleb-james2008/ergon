package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = 300
local S = B.STARTERS
local function cat(a, b) local d = { table.unpack(a) }; for _, x in ipairs(b) do d[#d+1] = x end; return d end
local decks = { {"S+Dojo (first deck)", cat(S, {"VG-HEAD-01"})}, {"S+Rare Bear", cat(S, {"VS-BEAR-01"})}, {"S+Rare Sea", cat(S, {"BC-SEA-01"})}, {"S+Epic Guts", cat(S, {"BK-GUTS-01"})}, {"4 Epics", {"BK-GUTS-01","MHA-BLAST-01","NR-SHADOW-01","VG-RIVAL-01"}}, {"3 starters only", S} }
for _, step in ipairs({1.5, 1.75, 2, 2.5}) do
  B.enemyLevel = function(i) return 1 + math.floor(step * i + 0.5) end
  local lv = {}; for i = 0, 11 do lv[#lv+1] = tostring(1 + math.floor(step * i + 0.5)) end
  print(string.format("\n== enemy level = 1 + round(%.2f * routeIndex): %s   (win%% at player max level, N=%d)", step, table.concat(lv, ","), N))
  print(string.format("%-20s e1  e2  e3  b1 | w2e1 w2e2 w2e3 b2 | w3e1 w3e2 w3e3 b3", ""))
  for _, d in ipairs(decks) do print(string.format("%-20s %s", d[1], B.fmt(B.row(d[2], 30, N)))) end
end
