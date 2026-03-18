# Roadmap

## Milestone 1 — Foundation

Get data flowing from the RS3 APIs through the Rust backend to a basic Vue frontend.

### rs3-api crate
- [ ] Set up `reqwest` + `tokio` dependencies
- [ ] Create `Rs3Client` struct with shared `reqwest::Client`
- [ ] Implement hiscores lite parser — map CSV positions to skills and activities
- [ ] Implement RuneMetrics profile fetcher — deserialize JSON into typed structs
- [ ] Implement RuneMetrics activity feed parser — extract boss kill events for auto-incrementing kill counts
- [ ] Implement Weird Gloop price fetcher — `/latest` endpoint for item prices
- [ ] Implement GE Database item detail fetcher — for item metadata and icons
- [ ] Define skill ID and activity ID mappings (position → name)
- [ ] Add error types for API failures (network, parse, not found, private profile)
- [ ] Write unit tests with sample API responses

### common-db crate
- [ ] Set up `mongodb` driver dependency
- [ ] Create `DbClient` struct with connection pooling
- [ ] Define MongoDB document structs (Boss, Drop, Goal, Settings) with serde
- [ ] Implement CRUD operations for bosses
- [ ] Implement CRUD operations for drops
- [ ] Implement CRUD operations for goals
- [ ] Implement settings read/write
- [ ] Connection string configuration (env var or config file)

### Tauri app wiring
- [ ] Add `rs3-api` and `common-db` as dependencies to `src-tauri`
- [ ] Create initial Tauri commands: `get_player_stats`, `get_boss_list`
- [ ] Set up Tauri state management (shared `Rs3Client` and `DbClient` instances)
- [ ] Install `@tauri-apps/api` in the Vue frontend

### Vue frontend — Boss Dashboard (basic)
- [ ] Remove Vue scaffold boilerplate (HelloWorld, Welcome, etc.)
- [ ] Set up Vue Router with routes: `/bosses`, `/goals`, `/settings`
- [ ] Set up app layout with sidebar/nav
- [ ] Create Pinia store for boss data
- [ ] Create `BossCard` component — displays boss name, kill count, wealth
- [ ] Create `BossDashboard` view — grid of boss cards
- [ ] Wire up to Tauri commands — fetch and display real data on load

---

## Milestone 2 — Boss Details & Drop Logging

Make the boss tracker fully functional with detail views and drop logging.

### Boss detail view
- [ ] Create `BossDetail` view — navigated to from boss card click
- [ ] Display full boss stats: kills, total wealth, significant drops list
- [ ] Show boss icon/image (from GE Database or RS Wiki)
- [ ] Back navigation to dashboard

### Drop logging
- [ ] Create `DropLogger` component — form to log a significant drop
- [ ] Item name input with autocomplete (search GE Database)
- [ ] Auto-fetch price from Weird Gloop when item is selected
- [ ] Allow manual price override
- [ ] Save drop to MongoDB and update boss wealth
- [ ] Create Tauri commands: `log_drop`, `add_misc_loot`

### Misc loot tracking
- [ ] Add GP input on boss detail view for misc loot
- [ ] Tauri command to add GP directly to boss `total_wealth`

### Wealth overview
- [ ] Create `WealthOverview` component — total PvM wealth across all bosses
- [ ] Per-boss wealth breakdown (could be on dashboard or separate view)

---

## Milestone 3 — Goals

Full goal tracking system.

### Goal backend
- [ ] Implement goal CRUD Tauri commands: `create_goal`, `update_goal`, `delete_goal`, `get_goals`
- [ ] Implement auto-update logic for numeric goals linked to boss/skill
- [ ] Calculate progress percentage for all goal types

### Goal frontend
- [ ] Create `GoalsDashboard` view — at-a-glance progress bars
- [ ] Create `GoalCard` component — shows title, type, progress
- [ ] Create `GoalDetail` view — full goal with editable checklist/numeric/checkbox
- [ ] Create `GoalForm` component — create/edit goals with type selection
- [ ] Checklist UI with nested items, expand/collapse, toggle completion
- [ ] Category filtering on the dashboard

---

## Milestone 4 — Analytics & Polish

Data visualization, advanced features, and cross-machine sync.

### Analytics
- [ ] Wealth over time chart (per boss and total) — requires timestamped wealth snapshots
- [ ] Drop rate luck calculator — input kills and drop rate, show probability
- [ ] Boss-specific achievement checklists (titles, pets, collection log items)

### Price comparison
- [ ] Show Weird Gloop price vs GE guide price side-by-side on drops
- [ ] Price history chart for items (using Weird Gloop `/last90d`)

### Settings & sync
- [ ] Settings view — configure RS3 display name, MongoDB connection, theme
- [ ] Settings stored in MongoDB for cross-machine sync
- [ ] Handle offline/disconnected gracefully (local cache, sync on reconnect)

### UI polish
- [ ] Loading states and error handling throughout
- [ ] Keyboard shortcuts for common actions
- [ ] Responsive layout for different window sizes
