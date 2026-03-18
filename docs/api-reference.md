# RS3 API Reference

All API calls are made from the Rust backend (`rs3-api` crate). The frontend never calls these directly because most official RS3 APIs lack proper CORS headers.

## Hiscores Lite

| | |
|---|---|
| **URL** | `https://secure.runescape.com/m=hiscore/index_lite.ws?player={name}` |
| **Method** | GET |
| **Auth** | None |
| **Response** | Plain text CSV |

### Response Format
Each line contains comma-separated values. Skills come first (30 entries, 3 values each: rank, level, XP), followed by activities (31 entries, 2 values each: rank, count).

```
104122,2993,834489804    ← Overall (rank, total level, total XP)
89086,106,27247418       ← Attack (rank, level, XP)
...                      ← remaining skills (30 total)
-1,0                     ← Bounty Hunter (rank, count)
42422,1212               ← BA Attacker (rank, count)
...                      ← remaining activities (31 total)
```

A value of `-1` means unranked (not enough activity to appear on hiscores).

### Activities (in order, positions 31-61)
Bounty Hunter, BH Rogues, Dominion Tower, Crucible, Castle Wars, BA Attacker, BA Defender, BA Collector, BA Healer, Duel Tournament, Mobilising Armies, Conquest, Fist of Guthix, GG Athletics, GG Resource Race, WE2 Armadyl Contribution, WE2 Bandos Contribution, WE2 Armadyl PvP, WE2 Bandos PvP, Heist Guard, Heist Robber, CFP 5 Games, AF15 Cow Tipping, AF15 Rat Kills, RuneScore, Clue Easy, Clue Medium, Clue Hard, Clue Elite, Clue Master, Leagues

### Notes
- The order of skills and activities is fixed but not documented in the response — must be mapped by position
- **RS3 hiscores does NOT include boss kill counts** — that is OSRS only
- Boss kills are tracked via RuneMetrics activity feed instead (see below)

---

## RuneMetrics

### Player Profile

| | |
|---|---|
| **URL** | `https://apps.runescape.com/runemetrics/profile/profile?user={name}&activities={count}` |
| **Method** | GET |
| **Auth** | None (public profiles only) |
| **Response** | JSON |

### Response Fields
```json
{
  "name": "Pastafartian",
  "rank": "104,122",
  "totalskill": 2993,
  "totalxp": 834995064,
  "combatlevel": 147,
  "loggedIn": "false",
  "skillvalues": [
    { "level": 120, "xp": 1043937466, "rank": 90548, "id": 15 }
  ],
  "activities": [
    {
      "date": "17-Mar-2026 20:13",
      "details": "I now have at least 34000000 experience points in the Thieving skill.",
      "text": "34000000XP in Thieving"
    }
  ]
}
```

### Monthly XP

| | |
|---|---|
| **URL** | `https://apps.runescape.com/runemetrics/xp-monthly?searchName={name}&skillid={id}` |
| **Method** | GET |
| **Auth** | None |
| **Response** | JSON — 12-month XP gain history |

### Notes
- `skillvalues[].xp` is multiplied by 10 (divide by 10 for actual XP)
- `activities` count defaults to 20, max unknown
- Less strict rate limiting than official Jagex APIs

---

## Weird Gloop Prices (RS Wiki)

The preferred price source — provides real trade prices with volume data.

| | |
|---|---|
| **Base URL** | `https://api.weirdgloop.org/exchange/history/rs/` |
| **Method** | GET |
| **Auth** | None |
| **Required** | Descriptive `User-Agent` header (e.g. `gielinor-toolkit/0.1.0`) |

### Endpoints

#### `/latest` — Current prices (up to 100 items)
```
GET /latest?id=21787|21790
```
Returns JSON object keyed by item ID:
```json
{
  "21787": { "id": 21787, "timestamp": "2026-03-17T...", "price": 1200000, "volume": 54 },
  "21790": { "id": 21790, "timestamp": "2026-03-17T...", "price": 950000, "volume": null }
}
```

#### `/last90d` — 90-day price history (1 item)
```
GET /last90d?id=21787
```
Returns array of `[unix_timestamp, price, volume]`.

#### `/all` — Full price history (1 item)
```
GET /all?id=21787
```

#### `/sample` — ~150 evenly-spaced historical prices (1 item)
```
GET /sample?id=21787
```

### Query Parameters
- `id` — item ID (pipe-separated for `/latest`)
- `name` — item name, case-sensitive (pipe-separated for `/latest`)

---

## GE Database (Jagex)

Useful for item search, metadata, and icons. For actual prices, prefer Weird Gloop.

| | |
|---|---|
| **Base URL** | `https://secure.runescape.com/m=itemdb_rs/api/` |
| **Method** | GET |
| **Auth** | None |

### Endpoints

#### `catalogue/detail.json?item={id}` — Item details
```json
{
  "item": {
    "icon": "https://...",
    "icon_large": "https://...",
    "id": 21787,
    "type": "Ammo",
    "typeIcon": "https://...",
    "name": "Ascension bolts",
    "description": "...",
    "current": { "trend": "neutral", "price": "1,234" },
    "today": { "trend": "positive", "price": "+5" }
  }
}
```

#### `catalogue/items.json?category={cat}&alpha={letter}&page={page}` — Browse items
Returns 12 items per page.

#### `graph/{id}.json` — 180-day price graph data

---

## RS Wiki API (MediaWiki)

| | |
|---|---|
| **URL** | `https://runescape.wiki/api.php` |
| **Method** | GET |
| **Auth** | None |

Standard MediaWiki API. Useful for:
- Boss page content (drop tables, mechanics, images)
- Item data and icons
- Structured data via Semantic MediaWiki queries

### Example — Get page content
```
GET https://runescape.wiki/api.php?action=parse&page=Arch-Glacor&format=json
```
