use serde::{Deserialize, Serialize};

/// Full player stats from the hiscores lite endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub skills: Vec<SkillEntry>,
    pub activities: Vec<HiscoreActivityEntry>,
}

/// A single skill from the hiscores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    pub skill: Skill,
    pub rank: i64,
    pub level: u16,
    pub xp: u64,
}

/// A single activity from the hiscores (minigames, clue scrolls, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiscoreActivityEntry {
    pub activity: HiscoreActivity,
    pub rank: i64,
    pub count: i64,
}

/// RuneMetrics player profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuneMetricsProfile {
    pub name: String,
    pub combat_level: u16,
    pub total_skill: u16,
    pub total_xp: u64,
    pub rank: Option<String>,
    pub logged_in: bool,
    pub skills: Vec<RuneMetricsSkill>,
    pub activities: Vec<Activity>,
}

/// A skill from the RuneMetrics profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuneMetricsSkill {
    pub id: u8,
    pub level: u16,
    pub xp: u64,
    pub rank: i64,
}

/// A recent activity from the RuneMetrics profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub date: String,
    pub text: String,
    pub details: String,
}

/// Tracks where we left off in the RuneMetrics activity feed
/// so we don't double-count boss kills between polls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCursor {
    pub last_activity_date: String,
    pub last_activity_text: String,
}

/// A parsed boss kill event from the RuneMetrics activity feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossKillEvent {
    pub boss_name: String,
    pub kill_count: u32,
    pub date: String,
}

/// Item price from the Weird Gloop API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPrice {
    pub id: u32,
    pub price: i64,
    pub volume: Option<i64>,
    pub timestamp: String,
}

/// Item details from the GE Database API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetail {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub icon_large_url: String,
    pub item_type: String,
    pub ge_price: String,
}

// ─────────────────────────────────────────────
// Skill and HiscoreActivity enums
//
// The hiscores lite endpoint returns skills and activities
// by position in the CSV, not by name. These enums map
// positions to names so we can work with typed data
// instead of raw indices.
//
// Note: RS3 hiscores does NOT include boss kill counts.
// Boss kills are tracked via RuneMetrics activity feed.
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Overall,
    Attack,
    Defence,
    Strength,
    Constitution,
    Ranged,
    Prayer,
    Magic,
    Cooking,
    Woodcutting,
    Fletching,
    Fishing,
    Firemaking,
    Crafting,
    Smithing,
    Mining,
    Herblore,
    Agility,
    Thieving,
    Slayer,
    Farming,
    Runecrafting,
    Hunter,
    Construction,
    Summoning,
    Dungeoneering,
    Divination,
    Invention,
    Archaeology,
    Necromancy,
}

impl Skill {
    /// Returns the skill at the given hiscores CSV position (0-indexed).
    pub fn from_index(index: usize) -> Option<Self> {
        SKILL_ORDER.get(index).copied()
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Overall => "Overall",
            Self::Attack => "Attack",
            Self::Defence => "Defence",
            Self::Strength => "Strength",
            Self::Constitution => "Constitution",
            Self::Ranged => "Ranged",
            Self::Prayer => "Prayer",
            Self::Magic => "Magic",
            Self::Cooking => "Cooking",
            Self::Woodcutting => "Woodcutting",
            Self::Fletching => "Fletching",
            Self::Fishing => "Fishing",
            Self::Firemaking => "Firemaking",
            Self::Crafting => "Crafting",
            Self::Smithing => "Smithing",
            Self::Mining => "Mining",
            Self::Herblore => "Herblore",
            Self::Agility => "Agility",
            Self::Thieving => "Thieving",
            Self::Slayer => "Slayer",
            Self::Farming => "Farming",
            Self::Runecrafting => "Runecrafting",
            Self::Hunter => "Hunter",
            Self::Construction => "Construction",
            Self::Summoning => "Summoning",
            Self::Dungeoneering => "Dungeoneering",
            Self::Divination => "Divination",
            Self::Invention => "Invention",
            Self::Archaeology => "Archaeology",
            Self::Necromancy => "Necromancy",
        }
    }
}

const SKILL_ORDER: &[Skill] = &[
    Skill::Overall,
    Skill::Attack,
    Skill::Defence,
    Skill::Strength,
    Skill::Constitution,
    Skill::Ranged,
    Skill::Prayer,
    Skill::Magic,
    Skill::Cooking,
    Skill::Woodcutting,
    Skill::Fletching,
    Skill::Fishing,
    Skill::Firemaking,
    Skill::Crafting,
    Skill::Smithing,
    Skill::Mining,
    Skill::Herblore,
    Skill::Agility,
    Skill::Thieving,
    Skill::Slayer,
    Skill::Farming,
    Skill::Runecrafting,
    Skill::Hunter,
    Skill::Construction,
    Skill::Summoning,
    Skill::Dungeoneering,
    Skill::Divination,
    Skill::Invention,
    Skill::Archaeology,
    Skill::Necromancy,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HiscoreActivity {
    BountyHunter,
    BhRogues,
    DominionTower,
    Crucible,
    CastleWars,
    BaAttacker,
    BaDefender,
    BaCollector,
    BaHealer,
    DuelTournament,
    MobilisingArmies,
    Conquest,
    FistOfGuthix,
    GgAthletics,
    GgResourceRace,
    We2ArmadylContribution,
    We2BandosContribution,
    We2ArmadylPvp,
    We2BandosPvp,
    HeistGuard,
    HeistRobber,
    Cfp5Games,
    Af15CowTipping,
    Af15RatKills,
    RuneScore,
    ClueEasy,
    ClueMedium,
    ClueHard,
    ClueElite,
    ClueMaster,
    Leagues,
}

impl HiscoreActivity {
    /// Returns the activity at the given position (0-indexed, after skills).
    pub fn from_index(index: usize) -> Option<Self> {
        ACTIVITY_ORDER.get(index).copied()
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::BountyHunter => "Bounty Hunter",
            Self::BhRogues => "BH Rogues",
            Self::DominionTower => "Dominion Tower",
            Self::Crucible => "Crucible",
            Self::CastleWars => "Castle Wars",
            Self::BaAttacker => "BA Attacker",
            Self::BaDefender => "BA Defender",
            Self::BaCollector => "BA Collector",
            Self::BaHealer => "BA Healer",
            Self::DuelTournament => "Duel Tournament",
            Self::MobilisingArmies => "Mobilising Armies",
            Self::Conquest => "Conquest",
            Self::FistOfGuthix => "Fist of Guthix",
            Self::GgAthletics => "GG Athletics",
            Self::GgResourceRace => "GG Resource Race",
            Self::We2ArmadylContribution => "WE2 Armadyl Contribution",
            Self::We2BandosContribution => "WE2 Bandos Contribution",
            Self::We2ArmadylPvp => "WE2 Armadyl PvP",
            Self::We2BandosPvp => "WE2 Bandos PvP",
            Self::HeistGuard => "Heist Guard",
            Self::HeistRobber => "Heist Robber",
            Self::Cfp5Games => "CFP 5 Games",
            Self::Af15CowTipping => "AF15 Cow Tipping",
            Self::Af15RatKills => "AF15 Rat Kills",
            Self::RuneScore => "RuneScore",
            Self::ClueEasy => "Clue Scrolls Easy",
            Self::ClueMedium => "Clue Scrolls Medium",
            Self::ClueHard => "Clue Scrolls Hard",
            Self::ClueElite => "Clue Scrolls Elite",
            Self::ClueMaster => "Clue Scrolls Master",
            Self::Leagues => "Leagues",
        }
    }
}

const ACTIVITY_ORDER: &[HiscoreActivity] = &[
    HiscoreActivity::BountyHunter,
    HiscoreActivity::BhRogues,
    HiscoreActivity::DominionTower,
    HiscoreActivity::Crucible,
    HiscoreActivity::CastleWars,
    HiscoreActivity::BaAttacker,
    HiscoreActivity::BaDefender,
    HiscoreActivity::BaCollector,
    HiscoreActivity::BaHealer,
    HiscoreActivity::DuelTournament,
    HiscoreActivity::MobilisingArmies,
    HiscoreActivity::Conquest,
    HiscoreActivity::FistOfGuthix,
    HiscoreActivity::GgAthletics,
    HiscoreActivity::GgResourceRace,
    HiscoreActivity::We2ArmadylContribution,
    HiscoreActivity::We2BandosContribution,
    HiscoreActivity::We2ArmadylPvp,
    HiscoreActivity::We2BandosPvp,
    HiscoreActivity::HeistGuard,
    HiscoreActivity::HeistRobber,
    HiscoreActivity::Cfp5Games,
    HiscoreActivity::Af15CowTipping,
    HiscoreActivity::Af15RatKills,
    HiscoreActivity::RuneScore,
    HiscoreActivity::ClueEasy,
    HiscoreActivity::ClueMedium,
    HiscoreActivity::ClueHard,
    HiscoreActivity::ClueElite,
    HiscoreActivity::ClueMaster,
    HiscoreActivity::Leagues,
];
