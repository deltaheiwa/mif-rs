use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Avatar {
	pub height: i32,

	pub url: String,

	pub width: i32,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
	pub category: String,

	pub level: i32,

	pub points: i32,

	pub points_next_level: i32,

	pub role_id: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameStats {
	pub achievements: Option<Vec<Achievement>>,

	pub exit_game_after_death_count: i32,

	pub exit_game_by_suicide_count: i32,

	pub games_killed_count: i32,

	pub games_survived_count: i32,

	pub solo_lose_count: i32,

	pub solo_win_count: i32,

	pub total_lose_count: i32,

	pub total_play_time_in_minutes: i32,

	pub total_tie_count: i32,

	pub total_win_count: i32,

	pub village_lose_count: i32,

	pub village_win_count: i32,

	pub voting_lose_count: i32,

	pub voting_win_count: i32,

	pub werewolf_lose_count: i32,

	pub werewolf_win_count: i32,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoleCards {
	pub ability_id1: Option<String>,

	pub ability_id2: Option<String>,

	pub ability_id3: Option<String>,

	pub ability_id4: Option<String>,

	pub rarity: Option<String>,

	pub role_id1: Option<String>,

	pub role_id2: Option<String>,

	pub role_id_base: Option<String>,

	pub role_ids_advanced: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WolvesvillePlayer {
	pub avatars: Option<Vec<Avatar>>,

	pub badge_ids: Option<Vec<String>>,

	pub clan_id: Option<String>,

	pub creation_time: Option<String>,

	pub equipped_avatar: Option<Avatar>,

	pub game_stats: GameStats,

	pub id: String,

	pub last_online: Option<String>,

	pub level: Option<i32>,

	pub personal_message: Option<String>,

	pub previous_username: Option<String>,

	pub profile_icon_color: String,

	pub profile_icon_id: String,

	pub ranked_season_best_rank: Option<i32>,

	pub ranked_season_max_skill: Option<i32>,

	pub ranked_season_played_count: Option<i32>,

	pub ranked_season_skill: Option<i32>,

	pub received_roses_count: Option<i32>,

	pub role_cards: Option<Vec<RoleCards>>,

	pub sent_roses_count: Option<i32>,

	pub status: String,

	pub timestamp: Option<DateTime<Utc>>,

	pub username: String,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WolvesvilleClan {
	pub creation_time: String,

	pub description: Option<String>,

	pub gems: Option<i32>,

	pub gold: Option<i32>,

	pub icon: String,

	pub icon_color: String,

	pub id: String,

	pub join_type: String,

	pub language: String,

	pub leader_id: String,

	pub member_count: i32,

	pub min_level: i32,

	pub name: String,

	pub quest_history_count: i32,

	pub tag: Option<String>,

	pub xp: i32,
}
