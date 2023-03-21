use std::collections::HashMap;
use std::io::ErrorKind::{InvalidData, InvalidInput};
use std::mem::transmute;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use futures::future;
use tokio::io;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;
use tokio::time::sleep;

use protocol::{Packet, WriteCwData};
use protocol::nalgebra::{Point2, Point3};
use protocol::packet::{*, Hit};
use protocol::packet::common::{CreatureId, Item};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::drops::Drop;
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::*;
use protocol::utils::constants::SIZE_ZONE;
use protocol::utils::io_extensions::{ReadPacket, WriteArbitrary, WritePacket};

use crate::addon::{Addons, enable_pvp, freeze_time};
use crate::server::creature::Creature;
use crate::server::creature_id_pool::CreatureIdPool;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;

mod creature_id_pool;
pub(crate) mod player;
mod handle_packet;
pub mod creature;
pub mod utils;

pub struct Server {
	id_pool: RwLock<CreatureIdPool>,
	pub players: RwLock<Vec<Arc<Player>>>,
	drops: RwLock<HashMap<Point2<i32>, Vec<Drop>>>,
	pub addons: Addons
}

impl Server {
	pub fn new() -> Self {
		Self {
			id_pool: RwLock::new(CreatureIdPool::new()),
			players: RwLock::new(Vec::new()),
			drops: RwLock::new(HashMap::new()),
			addons: Addons::new()
		}
	}

	pub async fn run(self) {
		self.id_pool.write().await.claim(); //reserve 0 for the server itself

		let listener = TcpListener::bind("0.0.0.0:12345").await.expect("unable to bind listening socket");

		self.addons.discord_integration.run(&self);
		freeze_time(&self);

		loop {
			let (stream, _) = listener.accept().await.unwrap();

			let self_static: &'static Server = unsafe { transmute(&self) }; //todo: scoped task
			tokio::spawn(async move {
				if let Err(_) = self_static.handle_new_connection(stream).await {
					//TODO: error logging
				}
			});
		}
	}

	async fn handle_new_connection(&self, stream: TcpStream) -> io::Result<()> {
		stream.set_nodelay(true).unwrap();
		let (mut reader, mut writer) = split_and_buffer(stream);

		check_version(&mut reader, &mut writer).await?;

		let assigned_id = self.id_pool.write().await.claim();
		let result = self.handle_new_player(assigned_id, reader, writer).await;
		self.id_pool.write().await.free(assigned_id);
		result
	}

	async fn handle_new_player(&self, assigned_id: CreatureId, mut reader: BufReader<OwnedReadHalf>, mut writer: BufWriter<OwnedWriteHalf>) -> io::Result<()> {
		writer.write_packet(&ConnectionAcceptance).await?;
		write_abnormal_creature_update(&mut writer, assigned_id).await?;

		let (full_creature_update, character) = read_character_data(&mut reader).await?;

		let new_player = Player::new(
			assigned_id,
			character,
			writer,
		);

		self.on_join(&new_player, full_creature_update).await?;//todo: character 2x updated

		let new_player_arc = Arc::new(new_player);
		self.players.write().await.push(new_player_arc.clone());

		let _ = self.read_packets_forever(&new_player_arc, reader).await
			.expect_err("impossible");

		self.remove_player(&new_player_arc).await;

		self.announce(format!("[-] {}", new_player_arc.character.read().await.name)).await;
		self.addons.anti_cheat.on_leave(&new_player_arc).await;
		self.addons.command_manager.on_leave(&new_player_arc).await;

		Ok(())
	}

	pub async fn broadcast<Packet: FromServer>(&self, packet: &Packet, player_to_skip: Option<&Player>)
		where BufWriter<OwnedWriteHalf>: WriteCwData<Packet>//todo: specialization could obsolete this
	{
		future::join_all(self.players.read().await.iter().filter_map(|player| {
			if let Some(pts) = player_to_skip && ptr::eq(player.as_ref(), pts) {
				return None;
			}

			Some(player.send_ignoring(packet))
		})).await;
	}

	pub async fn add_drop(&self, item: Item, position: Point3<i64>, rotation: f32) {
		let zone = position.xy().map(|scalar| (scalar / SIZE_ZONE) as i32);

		let mut drops_guard = self.drops.write().await;
		let zone_drops = drops_guard.entry(zone).or_insert(vec![]);
		zone_drops.push(Drop {
			item,
			position,
			rotation,
			scale: 0.1,
			unknown_a: 0,
			unknown_b: 0,
			droptime: 0
		});
		let mut zone_drops_copy = zone_drops.clone();
		zone_drops_copy[zone_drops.len() - 1].droptime = 500;
		drop(drops_guard);

		self.broadcast(&WorldUpdate {
			drops: vec![(zone, zone_drops_copy)],
			sounds: vec![Sound::at(position, Drop)],
			..Default::default()
		}, None).await;

		let server_static: &'static Server = unsafe { transmute(self) }; //todo: scoped task
		tokio::spawn(async move {
			sleep(Duration::from_millis(500)).await;
			server_static.broadcast(&WorldUpdate::from(Sound::at(position, DropItem)), None).await;
		});
	}

	///returns none if a player picks up an item they dropped in single player
	pub async fn remove_drop(&self, zone: Point2<i32>, item_index: usize) -> Option<Item> {
		let mut drops_guard = self.drops.write().await;
		let Some(zone_drops) = drops_guard.get_mut(&zone) else { return None; };
		let removed_drop = zone_drops.swap_remove(item_index);
		let zone_drops_owned = zone_drops.to_owned();
		if zone_drops.is_empty() {
			drops_guard.remove(&zone);
		}
		drop(drops_guard);

		self.broadcast(&WorldUpdate::from((zone, zone_drops_owned)), None).await;

		Some(removed_drop.item)
	}

	async fn remove_player(&self, player_to_remove: &Player) {
		{
			let mut players = self.players.write().await;
			let index = players.iter().position(|player| ptr::eq(player_to_remove, player.as_ref())).expect("player not found");
			players.swap_remove(index);
		};
		self.remove_creature(&player_to_remove.id).await;
	}

	async fn remove_creature(&self, creature_id: &CreatureId) {
		//this is a shortcut, as the creature technically still exists
		//the proper way to remove a creature requires updating all remaining creatures which is expensive on bandwidth
		self.broadcast(&CreatureUpdate {
			id: creature_id.to_owned(),
			health: Some(0.0), //makes the creature intangible
			affiliation: Some(Affiliation::Neutral), //ensures it doesnt show up on the map
			..Default::default()
		}, None).await;
	}

	async fn read_packets_forever(&self, source: &Player, mut reader: BufReader<OwnedReadHalf>) -> io::Result<()> {
		loop {
			//todo: copypasta
			match reader.read_id().await? {
				CreatureUpdate       ::ID => self.handle_packet(source, reader.read_packet::<CreatureUpdate       >().await?).await,
				CreatureAction       ::ID => self.handle_packet(source, reader.read_packet::<CreatureAction       >().await?).await,
				Hit                  ::ID => self.handle_packet(source, reader.read_packet::<Hit                  >().await?).await,
				StatusEffect         ::ID => self.handle_packet(source, reader.read_packet::<StatusEffect         >().await?).await,
				Projectile           ::ID => self.handle_packet(source, reader.read_packet::<Projectile           >().await?).await,
				ChatMessageFromClient::ID => self.handle_packet(source, reader.read_packet::<ChatMessageFromClient>().await?).await,
				ZoneDiscovery        ::ID => self.handle_packet(source, reader.read_packet::<ZoneDiscovery        >().await?).await,
				RegionDiscovery      ::ID => self.handle_packet(source, reader.read_packet::<RegionDiscovery      >().await?).await,
				unexpected_packet_id => panic!("unexpected packet id {:?}", unexpected_packet_id)
			};

			if source.should_disconnect.load(Ordering::Relaxed) {
				return Err(InvalidInput.into());
			}
		}
	}

	async fn on_join(&self, player: &Player, full_creature_update: CreatureUpdate) -> io::Result<()> {
		self.addons.anti_cheat.on_join(player).await;
		self.handle_packet(player, full_creature_update).await;

		if player.should_disconnect.load(Ordering::Relaxed) {//todo: this is very error prone. need proper kick logic asap
			return Err(InvalidInput.into());
		}

		player.send(&MapSeed(56345)).await?;
		player.notify("welcome to berld").await;

		for existing_player in self.players.read().await.iter() {
			let mut creature_update = existing_player.character.read().await.to_update(existing_player.id);
			enable_pvp(&mut creature_update);
			player.send(&creature_update).await?;
		}

		player.send(&WorldUpdate {
			drops: self.drops.read().await
				.clone()
				.into_iter()
				.collect(),
			..Default::default()
		}).await?;

		self.announce(format!("[+] {}", player.character.read().await.name)).await;

		Ok(())
	}
}

fn split_and_buffer(stream: TcpStream) -> (BufReader<OwnedReadHalf>, BufWriter<OwnedWriteHalf>) {
	let (read_half, write_half) = stream.into_split();

	let reader = BufReader::new(read_half);
	let writer = BufWriter::new(write_half);

	(reader, writer)
}

async fn check_version(reader: &mut impl ReadPacket, writer: &mut impl WritePacket<ProtocolVersion>) -> io::Result<()> {
	if reader.read_id().await? != ProtocolVersion::ID {
		return Err(InvalidData.into());
	}

	if reader.read_packet::<ProtocolVersion>().await?.0 != 3 {
		writer.write_packet(&ProtocolVersion(3)).await?;
		return Err(InvalidInput.into());
	}

	Ok(())
}

/// during new player setup the server needs to send an abnormal CreatureUpdate which:
/// * is not compressed (and lacks the size prefix used for compressed packets)
/// * has no bitfield indicating the presence of its properties
/// * falls 8 bytes short of representing a full creature
///
/// unfortunately it is impossible to determine which bytes are missing exactly,
/// as the only reference is pixxie from the vanilla server, which is almost completely zeroed.
/// the last non-zero bytes in pixxie are the equipped weapons, which are positioned correctly.
/// from that it can be deduced that the missing bytes belong to the last 3 properties.
/// it's probably a cut-off at the end resulting from an incorrectly sized buffer
async fn write_abnormal_creature_update<Writable: AsyncWrite + Unpin + Send>(writable: &mut Writable, assigned_id: CreatureId) -> io::Result<()> {
	writable.write_arbitrary(&CreatureUpdate::ID).await?;
	writable.write_arbitrary(&assigned_id).await?; //luckily the only thing the alpha client does with this data is acquiring its assigned CreatureId
	writable.write_all(&[0u8; 4456]).await?; //so we can simply zero out everything else and not worry about the missing bytes
	writable.flush().await
	//TODO: move this to protocol crate and construct this from an actual [CreatureUpdate]
}

async fn read_character_data(reader: &mut impl ReadPacket) -> io::Result<(CreatureUpdate, Creature)> {
	if reader.read_id().await? != CreatureUpdate::ID {
		return Err(InvalidData.into());
	}

	let creature_update = reader.read_packet::<CreatureUpdate>().await?;

	let Some(character) = Creature::maybe_from(&creature_update) else {
		return Err(InvalidInput.into());
	};

	Ok((creature_update, character))
}