package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = 1000
local S = B.STARTERS
local isStarter = {} for _, id in ipairs(S) do isStarter[id] = true end
local rows = {}
for id, e in pairs(B.by) do
  if not isStarter[id] then
    local d = { table.unpack(S) }; d[#d+1] = id
    rows[#rows+1] = { id = id, rarity = e.rarity, alias = e.alias, w = B.wins(d, 30, B.ROUTES[9], N) / N }
  end
end
local order = { Common = 1, Uncommon = 2, Rare = 3, Epic = 4, Legendary = 5, Mythic = 6 }
table.sort(rows, function(a, b) if order[a.rarity] ~= order[b.rarity] then return order[a.rarity] < order[b.rarity] end return a.id < b.id end)
local opens, total = {}, {}
for _, r in ipairs(rows) do
  print(string.format("%-10s %-18s %-18s %5.1f%%", r.rarity, r.id, r.alias, r.w * 100))
  total[r.rarity] = (total[r.rarity] or 0) + 1
  if r.w >= 0.9 then opens[r.rarity] = (opens[r.rarity] or 0) + 1 end
end
for _, k in ipairs({"Common","Uncommon","Rare","Epic","Legendary","Mythic"}) do print(k, (opens[k] or 0) .. "/" .. (total[k] or 0), "open the door (>=90%)") end
