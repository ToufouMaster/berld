use nalgebra::Point3;
use strum_macros::EnumIter;

use crate::packet::world_update::Sound;
use crate::utils::sound_position_of;

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum Kind {
	Hit,
	Blade1,
	Blade2,
	LongBlade1,
	LongBlade2,
	Hit1,
	Hit2,
	Punch1,
	Punch2,
	HitArrow,
	HitArrowCritical,
	Smash1,
	SlamGround,
	SmashHit2,
	SmashJump,
	Swing,
	ShieldSwing,
	SwingSlow,
	SwingSlow2,
	ArrowDestroy,
	Blade3,
	Punch3,
	Salvo2,
	SwordHit03,
	Block,
	ShieldSlam,
	Roll,
	Destroy2,
	Cry,
	Levelup2,
	Missioncomplete,
	Watersplash01,
	Step2,
	StepWater,
	StepWater2,
	StepWater3,
	Channel2,
	ChannelHit,
	Fireball,
	FireHit,
	Magic01,
	Watersplash,
	WatersplashHit,
	LichScream,
	Drink2,
	Pickup,
	Disenchant2,
	Upgrade2,
	Swirl,
	HumanVoice01,
	HumanVoice02,
	Gate,
	SpikeTrap,
	FireTrap,
	Lever,
	Charge2,
	Magic02,
	Drop,
	DropCoin,
	DropItem,
	MaleGroan,
	FemaleGroan,
	MaleGroan2,
	FemaleGroan2,
	GoblinMaleGroan,
	GoblinFemaleGroan,
	LizardMaleGroan,
	LizardFemaleGroan,
	DwarfMaleGroan,
	DwarfFemaleGroan,
	OrcMaleGroan,
	OrcFemaleGroan,
	UndeadMaleGroan,
	UndeadFemaleGroan,
	FrogmanMaleGroan,
	FrogmanFemaleGroan,
	MonsterGroan,
	TrollGroan,
	MoleGroan,
	SlimeGroan,
	ZombieGroan,
	Explosion,
	Punch4,
	MenuOpen2,
	MenuClose2,
	MenuSelect,
	MenuTab,
	MenuGrabItem,
	MenuDropItem,
	Craft,
	CraftProc,
	Absorb,
	Manashield,
	Bulwark,
	Bird1,
	Bird2,
	Bird3,
	Cricket1,
	Cricket2,
	Owl1,
	Owl2
}

impl Sound {
	#[must_use]
	pub fn at(position: Point3<i64>, kind: Kind) -> Self {
		Self {
			position: sound_position_of(position),
			kind,
			volume: 1.0,
			pitch: 1.0
		}
	}
}