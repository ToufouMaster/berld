use tap::Tap;

use protocol::packet::{CreatureUpdate, Hit, WorldUpdate};
use protocol::packet::common::Race;
use protocol::packet::common::item::Kind::Weapon;
use protocol::packet::common::item::kind::Weapon::Shield;
use protocol::packet::common::Race::*;
use protocol::packet::creature_update::equipment::Slot;
use protocol::packet::hit::Kind::{*, Absorb, Block};
use protocol::packet::world_update::{Sound, sound};
use protocol::packet::world_update::sound::Kind::*;

use crate::addon::balancing;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<Hit> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: Hit) {
		let players_guard = self.players.read().await;
		let Some(target) = players_guard.iter().find(|player| { player.id == packet.target }) else {

			let creatures_guard_read = self.creatures.read().await;
			let Some((id, target)) = &creatures_guard_read.clone().into_iter().find(|(id, creature)| { *id == packet.target }) else {
				return; //can happen when the target disconnected in this moment
			};
			drop(creatures_guard_read);
			let source_character_guard = source.character.read().await;
			let mut creatures_guard = self.creatures.write().await;
			let Some(mut target_writable) = creatures_guard.get_mut(id) else {return};

			balancing::adjust_hit(&mut packet, &source_character_guard, &target);
			packet.flash = true;//todo: (re-)move


			source.send_ignoring(&CreatureUpdate { // Avoid the depletion of the target blocking gauge
				id: *id,
				blocking_gauge: Some(target.blocking_gauge),
				..Default::default()
			}).await;

			let mut hits_vec = vec![];
			let mut hit_sounds = impact_sounds(&packet, target.race);

			if packet.kind == Block {
				let block_packet = Hit { // Show Block message when attack is Blocked
					kind: Block,
					damage: 0.0,
					critical: true, // text is clearer like this
					..packet
				};
				hits_vec.push(block_packet); // To target

				let left_weapon = &target.equipment[Slot::LeftWeapon];
				let right_weapon = &target.equipment[Slot::RightWeapon];
				if left_weapon.kind != Weapon(Shield) && right_weapon.kind != Weapon(Shield) { // No shield blocking
					packet.damage /= 4.0;
					packet.kind = Normal;
					hits_vec.push(packet); // Normal hit packet, but with damage divided by 4
				}
			} else {
				hits_vec.push(packet);
			}

			let mut next_health = target.health;
			for hit in &hits_vec {
				next_health -= hit.damage;
			}

			source.send_ignoring(&CreatureUpdate { // Avoid the depletion of the target blocking gauge
				id: *id,
				health: Some(next_health),
				..Default::default()
			}).await;
			target_writable.health = next_health;
			println!("{}", target_writable.health);

			if next_health <= 0.0 {
				self.broadcast(&WorldUpdate { // send death sound to all players
					sounds: vec![Sound::at(target.position, Destroy2)],
					..Default::default()
				}, None).await;
			}
			return; //can happen when the target disconnected in this moment
		};
		let target_character_guard = target.character.read().await;
		let source_character_guard = source.character.read().await;

		balancing::adjust_hit(&mut packet, &source_character_guard, &target_character_guard);
		packet.flash = true;//todo: (re-)move


		source.send_ignoring(&CreatureUpdate { // Avoid the depletion of the target blocking gauge
			id: target.id,
			blocking_gauge: Some(target_character_guard.blocking_gauge),
			..Default::default()
		}).await;

		let mut hits_vec = vec![];
		let mut hit_sounds = impact_sounds(&packet, target_character_guard.race);

		if packet.kind == Block {
			let block_packet = Hit { // Show Block message when attack is Blocked
				kind: Block,
				damage: 0.0,
				critical: true, // text is clearer like this
				..packet
			};
			hits_vec.push(block_packet); // To target

			let left_weapon = &target_character_guard.equipment[Slot::LeftWeapon];
			let right_weapon = &target_character_guard.equipment[Slot::RightWeapon];
			if left_weapon.kind != Weapon(Shield) && right_weapon.kind != Weapon(Shield) { // No shield blocking
				packet.damage /= 4.0;
				packet.kind = Normal;
				hits_vec.push(packet); // Normal hit packet, but with damage divided by 4
			}
		} else {
			hits_vec.push(packet);
		}

		let mut next_health = target_character_guard.health;
		for hit in &hits_vec {
			next_health -= hit.damage;
		}


		target.send_ignoring(&WorldUpdate {
			sounds: hit_sounds, 	// the sound and hit effect can be heard/seen by every players
			hits: hits_vec,			// the damages are only shown to the target
			..Default::default()	// and the attacker damage is precalculated by the client
		}).await;

		if next_health <= 0.0 {
			self.broadcast(&WorldUpdate { // send death sound to all players
				sounds: vec![Sound::at(target_character_guard.position, Destroy2)],
				..Default::default()
			}, None).await;
		}
	}
}

pub fn impact_sounds(hit: &Hit, target_race: Race) -> Vec<Sound> {
	match hit.kind {
		Block |
		Miss => vec![sound::Kind::Block],

		Absorb => vec![sound::Kind::Absorb],

		Dodge |
		Invisible => vec![],

		Normal => {
			vec![Punch1]
				.tap_mut(|v| {
					if let Some(groan) = groan_of(target_race) {
						v.push(groan);
					}
				})
		},
	}.into_iter()
		.map(|kind| Sound::at(hit.position, kind))
		.collect()
}

const fn groan_of(race: Race) -> Option<sound::Kind> {
	match race {
		ElfMale         => Some(MaleGroan),
		ElfFemale       => Some(FemaleGroan),
		HumanMale       => Some(MaleGroan2),
		HumanFemale     => Some(FemaleGroan2),
		GoblinMale      => Some(GoblinMaleGroan),
		GoblinFemale    => Some(GoblinFemaleGroan),
		LizardmanMale   => Some(LizardMaleGroan),
		LizardmanFemale => Some(LizardFemaleGroan),
		DwarfMale       => Some(DwarfMaleGroan),
		DwarfFemale     => Some(DwarfFemaleGroan),
		OrcMale         => Some(OrcMaleGroan),
		OrcFemale       => Some(OrcFemaleGroan),
		FrogmanMale     => Some(FrogmanMaleGroan),
		FrogmanFemale   => Some(FrogmanFemaleGroan),
		UndeadMale      => Some(UndeadMaleGroan),
		UndeadFemale    => Some(UndeadFemaleGroan),
		_ => None
	}
}