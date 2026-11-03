-- proof.lua — the SAME simulation run against two source trees (BAL_SRC): before the balance pass and after it.
-- Real resolver, real Battles/Config of that tree, seeds s*7919 for s = 1..N, player = side A, skill multiplier 1, rules of that tree.
package.path = "./?.lua;" .. package.path
local B = require("balance")
local N = tonumber(os.getenv("BAL_N")) or 1000
local S = B.STARTERS
local function cat(a, b) local d = { table.unpack(a) }; for _, x in ipairs(b) do d[#d+1] = x end; return d end
local names = {"enc-1","enc-2","enc-3","boss-1","w2-enc-1","w2-enc-2","w2-enc-3","boss-2","w3-enc-1","w3-enc-2","w3-enc-3","boss-3"}
print("source: " .. (os.getenv("BAL_SRC") or "default") .. "  N=" .. N .. "  ROUTE_ENEMY_LEVEL=" .. (B.Config.ROUTE_ENEMY_LEVEL and "present" or "absent (all routes level 1)"))
local function table_for(title, decks, level)
  print("\n### " .. title .. " (every card at level " .. level .. ", i.e. at its chapter ceiling; win % of " .. N .. " fights per cell)")
  print("| deck | " .. table.concat(names, " | ") .. " | routes won >= 90% |")
  print("|---|" .. string.rep("---|", 13))
  for _, d in ipairs(decks) do
    local r = B.row(d[2], level, N)
    local c, cells = 0, {}
    for i, x in ipairs(r) do cells[i] = string.format("%d", math.floor(x * 100 + 0.5)); if x >= 0.9 then c = c + 1 end end
    print("| " .. d[1] .. " | " .. table.concat(cells, " | ") .. " | " .. c .. "/12 |")
  end
end
table_for("A. The starter decks, by card level", { {"3 starters, L6", S} }, 6)
table_for("A. The starter decks, by card level", { {"3 starters, L10", S} }, 10)
table_for("A. The starter decks, by card level", { {"3 starters, L30 (ceiling)", S} }, 30)
local first = cat(S, {"VG-HEAD-01"})
table_for("B. The first deck (3 starters + free Dojo Teacher)", { {"first deck, L6", first} }, 6)
table_for("B. The first deck (3 starters + free Dojo Teacher)", { {"first deck, L10", first} }, 10)
table_for("B. The first deck (3 starters + free Dojo Teacher)", { {"first deck, L30 (ceiling)", first} }, 30)
table_for("C. Does collecting matter? starters + ONE collected card", {
  {"+ Goku (Uncommon)", cat(S, {"DB-GOKU-01"})}, {"+ Musashi (Uncommon)", cat(S, {"VG-MUSASHI-01"})},
  {"+ Sea (Rare)", cat(S, {"BC-SEA-01"})}, {"+ Ichigo (Rare)", cat(S, {"BL-ICHIGO-01"})}, {"+ Namek (Rare)", cat(S, {"DB-NAMEK-01"})}, {"+ Bear (Rare)", cat(S, {"VS-BEAR-01"})},
  {"+ Guts (Epic)", cat(S, {"BK-GUTS-01"})}, {"+ Kakashi (Legendary)", cat(S, {"NR-KAKASHI-01"})},
  {"4 Epics", {"BK-GUTS-01","MHA-BLAST-01","NR-SHADOW-01","VG-RIVAL-01"}},
}, 30)
