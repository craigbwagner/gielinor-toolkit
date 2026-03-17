# Architecture

## Overview

Gielinor Toolkit is a single Tauri v2 desktop app with Vue Router for navigation between features (boss tracker, goals, settings, etc.). The Rust backend is organized as a Cargo workspace with shared and feature-specific crates.

## Crate Responsibilities

### `src-tauri/` — Tauri App Shell
- Entry point for the desktop app
- Wires together all feature crates by registering their Tauri commands
- Manages app lifecycle, window config, and plugin setup
- Does not contain business logic — delegates to feature crates

### `crates/rs3-api/` — RS3 API Client (shared)
- Single `Rs3Client` struct holding a `reqwest::Client` for connection pooling
- Methods for each RS3 API endpoint (hiscores, RuneMetrics, Weird Gloop, GE Database)
- Parses raw API responses into typed Rust structs
- Used by feature crates to fetch player data and item prices
- All RS3 API calls go through this crate — the frontend never calls RS3 APIs directly (most lack CORS headers)

### `crates/common-db/` — MongoDB Layer (shared)
- MongoDB connection management
- Document models (Boss, Drop, Goal, Settings, etc.)
- CRUD operations and queries
- Used by feature crates for all persistence

### `crates/boss-tracker/` — Boss Tracking Feature
- Boss-related business logic
- Tauri commands: fetch boss data, log drops, add wealth, get boss details
- Depends on `rs3-api` and `common-db`

### `crates/goals/` — Goal Tracking Feature
- Goal-related business logic
- Tauri commands: CRUD goals, calculate progress, auto-update from API data
- Depends on `rs3-api` and `common-db`

## Data Flow

```
┌─────────────────────────────────────────────────────┐
│                    Vue Frontend                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │  Pinia   │  │  Vue     │  │   Components     │  │
│  │  Stores  │  │  Router  │  │   (BossCard,     │  │
│  │          │  │          │  │    GoalTracker,   │  │
│  │          │  │          │  │    DropLogger)    │  │
│  └────┬─────┘  └──────────┘  └──────────────────┘  │
│       │  invoke()                                    │
├───────┼─────────────────────────────────────────────┤
│       ▼         Tauri Command Bridge                 │
├─────────────────────────────────────────────────────┤
│                  Rust Backend                        │
│  ┌──────────────┐  ┌──────────────┐                 │
│  │ boss-tracker │  │    goals     │  feature crates  │
│  └──────┬───────┘  └──────┬───────┘                 │
│         │                 │                          │
│  ┌──────▼─────────────────▼───────┐                 │
│  │           rs3-api              │  shared crates   │
│  │         common-db              │                  │
│  └──────┬──────────────┬──────────┘                 │
│         │              │                             │
├─────────┼──────────────┼─────────────────────────────┤
          ▼              ▼
    RS3 APIs          MongoDB
```

1. **Frontend** calls Tauri commands via `invoke()` from Pinia stores
2. **Tauri commands** (defined in feature crates) handle business logic
3. **Feature crates** use `rs3-api` to fetch external data and `common-db` to persist/query
4. **Results** flow back to the frontend as serialized JSON via Tauri's command response
