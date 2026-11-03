package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = 500
local S = B.STARTERS
local isStarter = {} for _, id in ipairs(S) do isStarter[id] = true end
local worst = {}
for id, e in pairs(B.by) do
  if not isStarter[id] then
    local d = { table.unpack(S) }; d[#d+1] = id
    local r = B.row(d, 30, N)
    local c = 0; local low = {}
    for i, x in ipairs(r) do if x >= 0.9 then c = c + 1 else low[#low+1] = B.ROUTES[i][1] .. "=" .. math.floor(x*100+.5) end end
    print(string.format("%-10s %-18s %2d/12  %s", e.rarity, id, c, table.concat(low, " ")))
  end
end
