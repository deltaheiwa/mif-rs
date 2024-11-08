#[derive(Serialize, Deserialize, Debug)]
pub struct Avatar {
	pub height: i32,

	pub url: String,

	pub width: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Achievement {
	pub category: Option<String>,

	pub level: Option<i32>,

	pub points: Option<i32>,

	pub points_next_level: Option<i32>,

	pub role_id: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GameStats {
	pub achievements: Option<Vec<Achievement>>,

	pub exit_game_after_death_count: Option<i32>,

	pub exit_game_by_suicide_count: Option<i32>,

	pub games_killed_count: Option<i32>,

	pub games_survived_count: Option<i32>,

	pub solo_lose_count: Option<i32>,

	pub solo_win_count: Option<i32>,

	pub total_lose_count: Option<i32>,

	pub total_play_time_in_minutes: Option<i32>,

	pub total_tie_count: Option<i32>,

	pub total_win_count: Option<i32>,

	pub village_lose_count: Option<i32>,

	pub village_win_count: Option<i32>,

	pub voting_lose_count: Option<i32>,

	pub voting_win_count: Option<i32>,

	pub werewolf_lose_count: Option<i32>,

	pub werewolf_win_count: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug)]
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
pub struct Root {
	pub avatars: Option<Vec<Avatar>>,

	pub badge_ids: Option<Vec<String>>,

	pub clan_id: Option<String>,

	pub creation_time: Option<String>,

	pub equipped_avatar: Option<Avatar>,

	pub game_stats: Option<GameStats>,

	pub id: Option<String>,

	pub last_online: Option<String>,

	pub level: Option<i32>,

	pub personal_message: Option<String>,

	pub profile_icon_color: Option<String>,

	pub profile_icon_id: Option<String>,

	pub ranked_season_best_rank: Option<i32>,

	pub ranked_season_max_skill: Option<i32>,

	pub ranked_season_played_count: Option<i32>,

	pub ranked_season_skill: Option<i32>,

	pub received_roses_count: Option<i32>,

	pub role_cards: Option<Vec<RoleCards>>,

	pub sent_roses_count: Option<i32>,

	pub status: Option<String>,

	pub username: Option<String>,
}
use serde::{Serialize, Deserialize};
