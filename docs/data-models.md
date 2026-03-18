# Data Models

These are the core data models used across the Rust backend and stored in MongoDB.

## Boss

Represents a tracked boss with kill count and wealth data.

```rust
struct Boss {
    slug: String,                   // MongoDB _id, e.g. "arch-glacor"
    name: String,                   // Display name, e.g. "Arch-Glacor"
    kills: u32,                     // Current total kill count
    initial_kills: u32,             // Kill count when first added to the app
    total_wealth: u64,              // Running GP total (only tracks wealth since tracked_since)
    significant_drops: Vec<Drop>,   // Notable drops (uniques, rares)
    tracked_since: DateTime,        // When this boss was first added to the app
}
```

### Kill Count Tracking
- RS3 hiscores does **not** include boss kill counts (that's OSRS only)
- Initial kill count is set manually per boss (from the in-game boss log) and stored as `initial_kills`
- After setup, the app polls the RuneMetrics activity feed and parses "I killed X [boss]" entries to auto-increment `kills`
- The RuneMetrics feed only shows recent activity, so historical kills before app setup must be entered manually
- **Kills since tracking:** `kills - initial_kills` gives the number of kills tracked by the app

### Tracking Context
- `tracked_since` records when the boss was first added to the app
- `total_wealth` only reflects earnings since `tracked_since`, not lifetime earnings
- The UI can display: "1,500 kills (312 since tracking on 2026-03-18) — 1.2B earned since tracking"

### Wealth Tracking
- `total_wealth` is the single source of truth for GP earned from a boss
- When a significant drop is logged, its `gp_value` is added to `total_wealth` automatically
- Misc loot (non-significant drops) is added directly to `total_wealth` as a GP amount
- Total PvM wealth = sum of all bosses' `total_wealth`

## Drop

A significant drop — always a single notable item.

```rust
struct Drop {
    item_name: String,      // e.g. "Leng artefact"
    gp_value: u64,          // From Weird Gloop API or manual entry
    received_at: DateTime,  // When the drop was actually received (user-entered, defaults to now)
    logged_at: DateTime,    // When the drop was logged in the app (always automatic)
}
```

### Notes
- Drops are always quantity 1 (only notable/unique items are tracked individually)
- Non-significant loot is not logged as individual drops — just added as GP to the boss's `total_wealth`
- `received_at` lets users backfill drops they got before using the app, or log a drop from a previous session
- `logged_at` is always set automatically and is never user-editable

## Goal

Flexible goal tracking for any RS3 objective.

```rust
struct Goal {
    id: String,
    title: String,                  // e.g. "Completionist Cape", "500 Arch-Glacor kills"
    goal_type: GoalType,
    category: Option<String>,       // "PvM", "Collection Log", "Capes", "Skilling", etc.
    boss_slug: Option<String>,      // Links goal to a boss (for auto-update)
    skill_id: Option<u8>,           // Links goal to a skill (for auto-update)
    created_at: DateTime,
}
```

### GoalType

```rust
enum GoalType {
    Numeric { current: u64, target: u64 },
    Checkbox { completed: bool },
    Checklist { items: Vec<ChecklistItem> },
}
```

| Variant | Use Case | Example |
|---|---|---|
| `Numeric` | Progress toward a number | "500 Arch-Glacor kills" (current: 312, target: 500) |
| `Checkbox` | Done or not done | "Get Arch-Glacor pet" |
| `Checklist` | List of items to complete | "Zamorak collection log" |

### ChecklistItem

```rust
struct ChecklistItem {
    name: String,
    completed: bool,
    children: Option<Vec<ChecklistItem>>,  // Nested checklists
}
```

Checklists support arbitrary nesting. Example structure for a collection log:

```
Arch-Glacor Collection Log
├── ☐ Leng artefact
├── ☐ Dark ice shard
├── ☐ Dark ice sliver
├── ☐ Frozen core of Leng
└── Cosmetics
    ├── ☐ Glacor remnants
    └── ☐ Scripture of Wen
```

### Auto-Update Behavior
- If a goal has `boss_slug` set and is `Numeric`, `current` can auto-update from the boss's tracked kill count (which itself is updated via RuneMetrics activity feed)
- If a goal has `skill_id` set and is `Numeric`, `current` can auto-update from hiscores XP/level
- `Checkbox` and `Checklist` goals are always manually updated (RS3 doesn't expose collection log or pet data via API)

## TrackedActivity

Tracks a hiscores activity (clue scrolls, RuneScore, etc.) with the same "since tracking" context as bosses.

```rust
struct TrackedActivity {
    activity: HiscoreActivity,      // e.g. ClueHard, RuneScore
    count: i64,                     // Current count from hiscores
    initial_count: i64,             // Count when first tracked
    tracked_since: DateTime,        // When tracking started
}
```

### Notes
- Auto-updates from the hiscores API on each fetch
- **Count since tracking:** `count - initial_count`
- Example: "29 elite clues (20 since tracking on 2026-03-18)"

## ActivityCursor

Tracks where we left off in the RuneMetrics activity feed so we don't double-count boss kills between polls.

```rust
struct ActivityCursor {
    last_activity_date: String,   // e.g. "17-Mar-2026 20:13"
    last_activity_text: String,   // e.g. "I killed 33 Arch-Glacors."
}
```

### Polling Logic
1. Fetch the RuneMetrics activity feed
2. Walk entries from newest to oldest
3. Stop when we find the entry matching both `last_activity_date` and `last_activity_text`
4. Process everything above that point (the new entries) — parse boss kill events and increment counts
5. Update the cursor to the newest entry

### Edge Case
If the cursor's entry is no longer in the feed (too many new activities pushed it out), process everything in the feed and flag a warning to the user — "you've had a lot of activity since last sync, check your kill counts." The RuneMetrics feed returns a max of 20 activities, so this happens if the user has 20+ new activities between polls.

### Storage
The cursor is stored in the `settings` document in MongoDB alongside the player's display name and other config.

## MongoDB Collections

| Collection | Document Type | Key |
|---|---|---|
| `bosses` | Boss | `slug` |
| `drops` | Drop (with `boss_slug` field) | auto-generated |
| `goals` | Goal | `id` |
| `tracked_activities` | TrackedActivity | `activity` |
| `settings` | User preferences + ActivityCursor | single document |

### Notes
- Drops are stored in their own collection (not embedded in Boss documents) for easier querying and pagination
- The `bosses` collection stores the denormalized `significant_drops` for quick display, while the `drops` collection is the canonical source
- Settings is a single document containing user preferences (display name, theme, sync config, ActivityCursor, etc.)
