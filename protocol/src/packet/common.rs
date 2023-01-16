use crate::utils::flagset::FlagSet8;

use self::item::*;

pub mod item;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct CreatureId(pub i64);

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Race {
	ElfMale,
	ElfFemale,
	HumanMale,
	HumanFemale,
	GoblinMale,
	GoblinFemale,
	TerrierBull,
	LizardmanMale,
	LizardmanFemale,
	DwarfMale,
	DwarfFemale,
	OrcMale,
	OrcFemale,
	FrogmanMale,
	FrogmanFemale,
	UndeadMale,
	UndeadFemale,
	Skeleton,
	OldMan,
	Collie,
	ShepherdDog,
	SkullBull,
	Alpaca,
	AlpacaBrown,
	Egg,
	Turtle,
	Terrier,
	TerrierScottish,
	Wolf,
	Panther,
	Cat,
	CatBrown,
	CatWhite,
	Pig,
	Sheep,
	Bunny,
	Porcupine,
	SlimeGreen,
	SlimePink,
	SlimeYellow,
	SlimeBlue,
	Frightener,
	Sandhorror,
	Wizard,
	Bandit,
	Witch,
	Ogre,
	Rockling,
	Gnoll,
	GnollPolar,
	Monkey,
	Gnobold,
	Insectoid,
	Hornet,
	InsectGuard,
	Crow,
	Chicken,
	Seagull,
	Parrot,
	Bat,
	Fly,
	Midge,
	Mosquito,
	RunnerPlain,
	RunnerLeaf,
	RunnerSnow,
	RunnerDesert,
	Peacock,
	Frog,
	CreaturePlant,
	CreatureRadish,
	Onionling,
	OnionlingDesert,
	Devourer,
	Duckbill,
	Crocodile,
	CreatureSpike,
	Anubis,
	Horus,
	Jester,
	Spectrino,
	Djinn,
	Minotaur,
	NomadMale,
	NomadFemale,
	Imp,
	Spitter,
	Mole,
	Biter,
	Koala,
	Squirrel,
	Raccoon,
	Owl,
	Penguin,
	Werewolf,
	Santa,
	Zombie,
	Vampire,
	Horse,
	Camel,
	Cow,
	Dragon,
	BeetleDark,
	BeetleFire,
	BeetleSnout,
	BeetleLemon,
	Crab,
	CrabSea,
	Troll,
	TrollDark,
	Helldemon,
	Golem,
	GolemEmber,
	GolemSnow,
	Yeti,
	Cyclops,
	Mammoth,
	Lich,
	Runegiant,
	Saurian,
	Bush,
	BushSnow,
	BushSnowberry,
	PlantCotton,
	Scrub,
	ScrubCobweg,
	ScrubFire,
	Ginseng,
	Cactus,
	ChristmasTree,
	Thorntree,
	DepositGold,
	DepositIron,
	DepositSilver,
	DepositSandstone,
	DepositEmerald,
	DepositSapphire,
	DepositRuby,
	DepositDiamond,
	DepositIcecrystal,
	Scarecrow,
	Aim,
	Dummy,
	Vase,
	Bomb,
	FishSapphire,
	FishLemon,
	Seahorse,
	Mermaid,
	Merman,
	Shark,
	Bumblebee,
	Lanternfish,
	Mawfish,
	Piranha,
	Blowfish
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Item {
	pub type_major: TypeMajor,
	pub type_minor: TypeMinor,
	//pad 2
	pub seed: i32,
	pub recipe: TypeMajor,
	//pad 1
	pub minus_modifier: i16,//todo: structure alignment entails this properties' existence, name adopted from cuwo
	pub rarity: Rarity,
	pub material: Material,
	pub flags: FlagSet8<ItemFlag>,
	//pad1
	pub level: i16,
	//pad2
	pub spirits: [Spirit; 32],
	pub spirit_counter: i32
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Hitbox {
	///horizontal size in west/east direction. Note: this also scales the creature visually (whether this is a bug or intended behaviour is unclear)
	pub width: f32,
	///horizontal size in north/south direction
	pub depth: f32,
	///vertical size
	pub height: f32
}

//todo: find a crate for this
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EulerAngles {
	pub pitch: f32,
	pub roll: f32,
	pub yaw: f32
}