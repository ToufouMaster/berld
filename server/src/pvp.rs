use protocol::packet::creature_update::{CreatureFlag, CreatureUpdate};

pub fn enable_pvp(creature_update: &mut CreatureUpdate) {
	if let Some(ref mut flags) = creature_update.flags {
		flags.set(CreatureFlag::FriendlyFire, true)
	}
}