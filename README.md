# Gielinor Toolkit

A desktop companion app for RuneScape 3, focused on PvM tracking, drop logging, wealth tracking, and goal management. Built with Rust, Tauri v2, and Vue 3.

## Features (Planned)

### Boss Tracker

**Dashboard**
- Grid of boss cards, each showing the boss name, kill count, and total wealth earned
- Kill counts are set manually once per boss (initial count from in-game boss log), then auto-incremented by parsing the RuneMetrics activity feed for new kills

**Boss Detail View**
- Click any boss card to open a detailed view for that boss
- Full stats: kill count, total wealth earned, and a list of all significant drops logged
- Boss icon/image sourced from the RS Wiki
- Quick actions to log a drop or add misc loot GP

**Significant Drop Logging**
- Log notable drops (uniques, rares) individually — always quantity 1
- Auto-fetches the item's current trade price from the Weird Gloop API when you select an item
- Manual price override if the auto price isn't accurate
- Item search with autocomplete powered by the GE Database
- Each drop records both `received_at` (user-entered, defaults to now — supports backfilling older drops) and `logged_at` (automatic, when it was entered in the app)
- Logging a significant drop automatically adds its GP value to the boss's total wealth

**Misc Loot Tracking**
- For non-significant drops (commons, supplies, etc.), add a GP amount directly to a boss's total wealth without logging individual items
- Example: "I made 50m in commons at Arch-Glacor today" → add 50m to Arch-Glacor's wealth

**Wealth Tracking**
- Total wealth earned per boss (significant drops + misc loot combined into one number)
- Total PvM wealth across all bosses
- Per-boss wealth breakdown view

**Price Comparison**
- Display Weird Gloop real trade price alongside the GE guide price for drops
- Price history charts using Weird Gloop's 90-day history data

**Drop Rate Luck Calculator**
- Input your kill count and an item's known drop rate
- Calculates the probability of going that dry (e.g. "1,200 kills at 1/1,000 — 30% chance of being this unlucky")
- Drop rates sourced from the RS Wiki or the official RS3 drop rates page

**Boss Achievement Checklists**
- Track boss-specific achievements: titles, pets, collection log items
- Pre-populated from RS Wiki data where possible

### Goals

**Goal Types**
- **Numeric** — track progress toward a target number. Examples: "500 Arch-Glacor kills", "1B GP from PvM", "120 Necromancy". Shows a progress bar with current/target.
- **Checkbox** — simple done/not done toggle. Examples: "Get Arch-Glacor pet", "Unlock City of Senntisten".
- **Checklist** — a list of items to complete, with support for nested sub-checklists. Examples: "Zamorak collection log" (with each unique drop as an item), "Completionist cape" (with top-level categories like "All quests", "All skill requirements", each containing their own nested checklists).

**Linking Goals to Game Data**
- Goals can optionally link to a boss (via slug) or a skill (via skill ID)
- Linked numeric goals auto-update their progress from the hiscores API — e.g. a "500 Arch-Glacor kills" goal updates `current` automatically when fresh kill data is fetched
- Checkbox and checklist goals are manually updated (RS3 doesn't expose collection log or pet data via API)

**Goals Dashboard**
- At-a-glance view of all goals with progress bars and completion status
- Filter by category: PvM, Collection Log, Capes, Skilling, etc.
- Quick visual indication of what's close to completion

### Settings & Sync
- All data (bosses, drops, goals, settings) persisted in MongoDB
- Settings sync across machines via a shared MongoDB cluster — your data follows you between desktop and laptop
- Configurable RS3 display name, MongoDB connection, and UI preferences

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop framework | [Tauri v2](https://v2.tauri.app/) |
| Backend | Rust |
| Frontend | Vue 3 (Composition API, TypeScript) |
| State management | Pinia |
| Routing | Vue Router |
| Database | MongoDB |
| RS3 data | Hiscores, RuneMetrics, Weird Gloop Prices, GE Database, RS Wiki |

## Project Structure

```
gielinor-toolkit/
├── Cargo.toml              # Rust workspace root
├── src-tauri/              # Tauri app (Rust backend, commands, state)
├── ui/                     # Vue 3 frontend
├── crates/
│   ├── rs3-api/            # Shared: RS3 API client
│   ├── common-db/          # Shared: MongoDB connection, models, queries
│   ├── boss-tracker/       # Feature: boss tracking logic, Tauri commands
│   └── goals/              # Feature: goal tracking logic, Tauri commands
├── docs/                   # Project documentation
```

## Documentation

- [Architecture](docs/architecture.md) — project structure, crate responsibilities, data flow
- [API Reference](docs/api-reference.md) — RS3 APIs, endpoints, formats, and usage notes
- [Data Models](docs/data-models.md) — Rust structs and MongoDB document shapes
- [Roadmap](docs/roadmap.md) — milestones with detailed task breakdowns
- [Future Ideas](docs/future-ideas.md) — planned integrations and features without a milestone yet

## Development

### Prerequisites
- Rust (1.77.2+)
- Node.js (22+, via nvm)
- Tauri CLI v2: `cargo install tauri-cli --version ^2`
- A MongoDB instance (local or Atlas)

### Running
```bash
nvm use 22
cd ui && npm install && cd ..
cargo tauri dev
```
