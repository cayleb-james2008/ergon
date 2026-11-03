package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = tonumber(arg[2]) or 20
local ids = {}
for id, e in pairs(B.by) do ids[#ids+1] = id end
table.sort(ids)
local rank = { Common = 1, Uncommon = 2, Rare = 3, Epic = 4, Legendary = 5, Mythic = 6 }
local rname = { "Common", "Uncommon", "Rare", "Epic", "Legendary", "Mythic" }
local route = B.ROUTES[9]                       -- w3-enc-1 (index 8)
B.routeIndex = 8
local levels = {}
for s in (arg[1] or "1,6,8,9,10"):gmatch("[^,]+") do levels[#levels+1] = tonumber(s) end
local function winrate(deck)
  local w = 0
  for s = 1, N do if B.fight(deck, 30, route[2], route[3], s * 7919).snapshot.winner == "A" then w = w + 1 end end
  return w / N
end
for _, L in ipairs(levels) do
  B.enemyLevel = function(i) return (i == 8) and L or 1 end
  local pass, total, byTop = {}, {}, {}
  local bestLow, bestLowDeck = -1, nil
  local n = #ids
  for a = 1, n - 3 do for b = a + 1, n - 2 do for c = b + 1, n - 1 do for d = c + 1, n do
    local deck = { ids[a], ids[b], ids[c], ids[d] }
    local top = 0
    for _, id in ipairs(deck) do top = math.max(top, rank[B.by[id].rarity]) end
    local w = winrate(deck)
    total[top] = (total[top] or 0) + 1
    if w >= 0.9 then pass[top] = (pass[top] or 0) + 1 end
    if top <= 2 and w > bestLow then bestLow, bestLowDeck = w, table.concat(deck, ",") end
  end end end end
  local parts = {}
  for r = 1, 6 do if total[r] then parts[#parts+1] = string.format("%s-top:%d/%d", rname[r], pass[r] or 0, total[r]) end end
  print(string.format("w3-enc-1 enemy level %2d | 4-card decks (max level) winning >=90%% by the RAREST card held: %s | best Common/Uncommon-only deck %.0f%% (%s)", L, table.concat(parts, "  "), bestLow * 100, bestLowDeck))
end
