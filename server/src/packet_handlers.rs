use std::io;
use std::sync::Arc;

use parking_lot::lock_api::RawRwLockDowngrade;

use protocol::packet::*;
use protocol::packet::creature_action::CreatureActionType;
use protocol::packet::world_update::Pickup;

use crate::player::Player;
use crate::pvp::enable_pvp;
use crate::Server;
use crate::traffic_filter::filter;

pub fn on_creature_update(server: &Arc<Server>, source: &Arc<Player>, mut packet: CreatureUpdate) -> Result<(), io::Error> {
	enable_pvp(&mut packet);

	let mut character = source.creature.write();
	let snapshot = character.clone();
	character.update(&packet);
	unsafe { source.creature.raw().downgrade(); }//todo: not sure

	if filter(&mut packet, &snapshot, &character) {
		server.broadcast(&packet, Some(source));
	}

	Ok(())
}

pub fn on_creature_action(server: &Arc<Server>, source: &Arc<Player>, packet: CreatureAction) -> Result<(), io::Error> {
	let mut reimburse_item = false;
	match packet.type_ {
		CreatureActionType::Bomb => {
			source.notify("bombs are disabled".to_owned());
			reimburse_item = true;
		}
		CreatureActionType::Talk => {
			source.notify("quests coming soon(tm)".to_owned());
		}
		CreatureActionType::ObjectInteraction => {
			source.notify("object interactions are disabled".to_owned());
		}
		CreatureActionType::PickUp => {
			source.notify("ground items aren't implemented yet".to_owned());
		}
		CreatureActionType::Drop => {
			source.notify("ground items aren't implemented yet".to_owned());
			reimburse_item = true;
		}
		CreatureActionType::CallPet => {
			//source.notify("pets are disabled".to_owned());
		}
	}

	if reimburse_item {
		source.send(&WorldUpdate {
			pickups: vec![Pickup {
				interactor: source.creature.read().id,
				item: packet.item
			}],
			..Default::default()
		});
	}

	Ok(())
}

pub fn on_hit(server: &Arc<Server>, source: &Arc<Player>, packet: Hit) -> Result<(), io::Error> {
	server.broadcast(&WorldUpdate {
		hits: vec![packet],
		..Default::default()
	}, Some(source));

	Ok(())
}

pub fn on_status_effect(server: &Arc<Server>, source: &Arc<Player>, packet: StatusEffect) -> Result<(), io::Error> {
	server.broadcast(
		&WorldUpdate {
			status_effects: vec![packet],
			..Default::default()
		},
		Some(source)
	);

	Ok(())
}

pub fn on_projectile(server: &Arc<Server>, source: &Arc<Player>, packet: Projectile) -> Result<(), io::Error> {
	server.broadcast(
		&WorldUpdate {
			projectiles: vec![packet],
			..Default::default()
		},
		Some(source)
	);

	Ok(())
}

pub fn on_chat_message(server: &Arc<Server>, source: &Arc<Player>, packet: ChatMessageFromClient) -> Result<(), io::Error> {
	server.broadcast(
		&ChatMessageFromServer {
			source: source.creature.read().id,
			text: packet.text
		},
		None
	);

	Ok(())
}

pub fn on_current_chunk(_server: &Arc<Server>, _source: &Arc<Player>, _packet: CurrentChunk) -> Result<(), io::Error> {
	Ok(())
}

pub fn on_current_biome(_server: &Arc<Server>, _source: &Arc<Player>, _packet: CurrentBiome) -> Result<(), io::Error> {
	Ok(())
}