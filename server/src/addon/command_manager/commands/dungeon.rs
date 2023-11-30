use std::collections::HashMap;
use std::str::SplitWhitespace;
use protocol::nalgebra::{Point2, Point3};
use protocol::packet::world_update::Block;
use protocol::packet::world_update::block::Kind::Solid;
use protocol::packet::{CreatureUpdate, WorldUpdate};
use protocol::packet::common::{CreatureId, Hitbox};
use protocol::packet::common::Race::Gnobold;
use protocol::packet::creature_update::Affiliation::Enemy;
use protocol::packet::creature_update::{Animation, Appearance, AppearanceFlag, Multipliers, Occupation, Specialization};
use protocol::packet::creature_update::multipliers::Multiplier;
use protocol::rgb::RGB;
use protocol::utils::constants::SIZE_BLOCK;
use protocol::utils::flagset::FlagSet;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::{Dungeon, Xp};
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::creature::Creature;
use crate::server::player::Player;
use crate::server::Server;

pub enum FloorType {
	Floor,
	Wall,
}
pub type FloorMap = HashMap<Point2<i32>, FloorType>;

impl Dungeon {
	pub fn new() -> Self {
		Self {
			..Default::default()
		}
	}
}
impl Command for Dungeon {
	const LITERAL: &'static str = "dungeon";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, _params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;
		let character_guard = caller.character.read().await;
		let position = character_guard.position;

		// Room generation

		let mut floor_blocks = vec![];
		let mut room = FloorMap::new();
		for x in 0..100 {
			for y in 0..100 {
				floor_blocks.push(Block {
					position: Point3::new(((position[0] / SIZE_BLOCK) + x) as i32, ((position[1] / SIZE_BLOCK) + y) as i32, (position[2] / SIZE_BLOCK) as i32),
					color: RGB {r: (65 - (((x + y) % 2) * 10)) as u8, g: 50, b: 60},
					kind: Solid,
					padding: 0,
				});
				room.insert(Point2::new(((position[0] / SIZE_BLOCK) + x) as i32, ((position[1] / SIZE_BLOCK) + y) as i32), FloorType::Floor);
			}
		}

		let mut rooms_map_guard = self.rooms_map.write().await;
		rooms_map_guard.push(room);

		server.broadcast(&WorldUpdate {
			blocks: floor_blocks,
			..Default::default()
		}, None).await;

		// Creature spawn
		let id = server.id_pool.write().await.claim().0;
		let mut appearance_flag: FlagSet<u16, AppearanceFlag> = FlagSet::default();
		let mut multipliers = character_guard.multipliers.clone();
		multipliers[Multiplier::Health] = 20.0;
		let creature = Creature {
			position: Point3::new(character_guard.position[0], character_guard.position[1], character_guard.position[2]+(2*SIZE_BLOCK)),
			rotation: Default::default(),
			velocity: Default::default(),
			acceleration: Default::default(),
			velocity_extra: Default::default(),
			head_tilt: 0.0,
			flags_physics: Default::default(),
			affiliation: Enemy,
			race: Gnobold,
			animation: Animation::Idle,
			animation_time: 0,
			combo: 0,
			combo_timeout: 0,
			appearance: Appearance {
				head_model: 75,
				head_size: 1.01,
				head_offset: Point3::new(0.0, 0.5, 5.0),
				creature_size: Hitbox { width: 0.80, depth: 0.80, height: 1.80},
				flags: appearance_flag,
				..Default::default()
			},
			flags: Default::default(),
			effect_time_dodge: 0,
			effect_time_stun: 0,
			effect_time_fear: 0,
			effect_time_chill: 0,
			effect_time_wind: 0,
			show_patch_time: 0,
			occupation: Occupation::Warrior,
			specialization: Specialization::Default,
			level: 10,
			experience: 0,
			master: Default::default(),
			unknown36: 0,
			rarity: 0,
			unknown38: 0,
			home_zone: Default::default(),
			home: Default::default(),
			zone_to_reveal: Default::default(),
			unknown42: 0,
			name: "Default monster".to_string(),
			unknown24: [0.0, 0.0, 0.0],
			unknown25: [0.0, 0.0, 0.0],
			aim_displacement: Default::default(),
			health: 9999.0,
			mana: 0.0,
			blocking_gauge: 0.0,
			multipliers: multipliers,
			unknown31: 0,
			equipment: character_guard.equipment.clone(),
			skill_tree: character_guard.skill_tree.clone(),
			mana_charge: 0.0,
			unknown32: 0,
			consumable: Default::default(),
			mana_cubes: 0,
		};
		let creature_update = CreatureUpdate {
			id: CreatureId { 0: id },
			position: Some(creature.position.clone()),
			affiliation: Some(creature.affiliation.clone()),
			race: Some(creature.race.clone()),
			appearance: Some(creature.appearance.clone()),
			occupation: Some(creature.occupation.clone()),
			specialization: Some(creature.specialization.clone()),
			level: Some(creature.level.clone()),
			name: Some(creature.name.clone()),
			..Default::default()
		};
		server.broadcast(&creature_update, None).await;
		let mut creatures = server.creatures.write().await;
		creatures.insert(CreatureId { 0: id }, creature);

		Ok(None)
	}
}