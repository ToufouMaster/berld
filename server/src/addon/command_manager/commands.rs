use std::collections::HashMap;
use tokio::sync::RwLock;

use protocol::nalgebra::Point3;
use crate::addon::command_manager::commands::dungeon::FloorMap;
use crate::server::creature::Creature;

mod xp;
mod warp;
mod level;
mod countdown;
mod who;
mod gear;
mod kick;
mod player;
mod tp;
mod test;
mod dungeon;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Who;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct WhoIp;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Player;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Xp;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Level;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Countdown;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Warp {
	locations: HashMap<String, Point3<i64>>
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Gear;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Kick;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Tp;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Test;

#[derive(Default)]
pub struct Dungeon {
	rooms_map: RwLock<Vec<FloorMap>>,
}