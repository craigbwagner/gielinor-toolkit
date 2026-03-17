# Data Models

These are the core data models used across the Rust backend and stored in MongoDB.

## Boss

Represents a tracked boss with kill count and wealth data.

```rust
struct Boss {
    slug: String,                   // MongoDB _id, e.g. "arch-glacor"
    name: String,                   // Display name, e.g. "Arch-Glacor"
    kills: u32,                     // From hiscores API
    total_wealth: u64,              // Running GP total
    significant_drops: Vec<Drop>,   // Notable drops (uniques, rares)
}
```

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
- If a goal has `boss_slug` set and is `Numeric`, `current` can auto-update from hiscores kill count
- If a goal has `skill_id` set and is `Numeric`, `current` can auto-update from hiscores XP/level
- `Checkbox` and `Checklist` goals are always manually updated (RS3 doesn't expose collection log or pet data via API)

## MongoDB Collections

| Collection | Document Type | Key |
|---|---|---|
| `bosses` | Boss | `slug` |
| `drops` | Drop (with `boss_slug` field) | auto-generated |
| `goals` | Goal | `id` |
| `settings` | User preferences | single document |

### Notes
- Drops are stored in their own collection (not embedded in Boss documents) for easier querying and pagination
- The `bosses` collection stores the denormalized `significant_drops` for quick display, while the `drops` collection is the canonical source
- Settings is a single document containing user preferences (display name, theme, sync config, etc.)
