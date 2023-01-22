use async_trait::async_trait;

use protocol::packet::CreatureUpdate;

use crate::addons::{anti_cheat, fix_cutoff_animations};
use crate::addons::enable_pvp;
use crate::addons::traffic_filter::filter;
use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<CreatureUpdate> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: CreatureUpdate) {
		let mut character = source.creature.write().await;
		let snapshot = character.clone();
		character.update(&packet);
		drop(character);
		let character = source.creature.read().await;//todo: downgrade character lock

		if let Err(message) = anti_cheat::inspect_creature_update(&packet, &snapshot, &character) {
			dbg!(&message);
			self.kick(source, message).await;
			return;
		}

		enable_pvp(&mut packet);

		if filter(&mut packet, &snapshot, &character) {
			fix_cutoff_animations(&mut packet, &snapshot);
			self.broadcast(&packet, Some(source)).await;
		}
	}
}