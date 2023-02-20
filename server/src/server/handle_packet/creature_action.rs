use protocol::nalgebra::Vector3;
use protocol::packet::{creature_action, CreatureAction, WorldUpdate};
use protocol::packet::creature_action::Kind::*;
use protocol::packet::world_update::{Pickup, sound, Sound};
use protocol::utils::constants::SIZE_BLOCK;
use protocol::utils::sound_position_of;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<CreatureAction> for Server {
	async fn handle_packet(&self, source: &Player, packet: CreatureAction) {
		match packet.kind {
			creature_action::Kind::Bomb => {
				source.notify("bombs are disabled").await;

				//the player consumed a bomb, so we need to reimburse it
				let pickup = Pickup {
					interactor: source.id,
					item: packet.item
				};
				source.send_ignoring(&WorldUpdate::from(pickup)).await;
			}
			Talk => {
				source.notify("quests coming soon(tm)").await;
			}
			ObjectInteraction => {
				source.notify("object interactions are disabled").await;
			}
			PickUp => {
				if let Some(item) = self.remove_drop(packet.zone, packet.item_index as usize).await {
					let pickup = Pickup {
						interactor: source.id,
						item
					};
					let sound = Sound {
						position: sound_position_of(source.creature.read().await.position),
						kind: sound::Kind::Pickup,
						pitch: 1f32,
						volume: 1f32
					};
					let world_update = WorldUpdate {
						pickups: vec![pickup],
						sounds: vec![sound],
						..Default::default()
					};
					source.send_ignoring(&world_update).await;
				}
			}
			Drop => {
				let creature_guard = source.creature.read().await;

				self.add_drop(
					packet.item,
					creature_guard.position - Vector3::new(0, 0, SIZE_BLOCK),
					creature_guard.rotation.yaw
				).await;
			}
			CallPet => {
				//source.notify("pets are disabled".to_owned());
			}
		}
	}
}