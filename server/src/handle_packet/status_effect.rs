use std::time::Duration;

use async_trait::async_trait;
use tokio::time::sleep;

use protocol::nalgebra::Vector3;
use protocol::packet::{Hit, StatusEffect, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::hit::HitType;
use protocol::packet::status_effect::StatusEffectType;
use protocol::packet::world_update::sound_effect::Sound;
use protocol::packet::world_update::SoundEffect;
use protocol::utils::sound_position_of;

use crate::addons::balancing;
use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<StatusEffect> for Server {
	async fn handle_packet(&self, source: &Player, packet: StatusEffect) {
		match packet.type_ {
			StatusEffectType::Poison => {
				let players_guard = self.players.read().await; //todo: do i really have to do this?

				let Some(target) = players_guard.iter().find(|player| { player.id == packet.target }) else {
					return; //todo: invalid input? //can happen when the target disconnected in this moment
				};
				let target_owned = target.to_owned();
				let packet_clone = packet.clone();
				tokio::spawn(async move {
					apply_poison(&target_owned, &packet_clone).await;
				});
			}
			StatusEffectType::WarFrenzy => {
				balancing::buff_warfrenzy(&packet, self).await;
			}
			_ => ()
		}


		self.broadcast(&WorldUpdate::from(packet), Some(source)).await;
	}
}

async fn apply_poison(target: &Player, status_effect: &StatusEffect) {
	let tick_count = status_effect.duration / 500;

	for i in 0..=tick_count {
		if i != 0 {
			sleep(Duration::from_millis(500)).await;
		}

		let target_position = target.creature.read().await.position;

		let hit = Hit {
			attacker: CreatureId(0),//todo: check if this matters
			target: status_effect.target,
			damage: status_effect.modifier,
			critical: false,
			stuntime: 0,
			position: target_position,
			direction: Vector3::zeros(),
			is_yellow: false,
			type_: HitType::Normal,
			flash: true,
		};

		let sound_effect = SoundEffect {
			position: sound_position_of(target_position),
			sound: Sound::Absorb,
			pitch: 1.0,
			volume: 1.0
		};

		let world_update = WorldUpdate {
			hits: vec![hit],
			sound_effects: vec![sound_effect],
			..Default::default()
		};

		if let Err(_) = target.send(&world_update).await {
			//disconnects are handled in the reading thread
			break;
		};
	}
}