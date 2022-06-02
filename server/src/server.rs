use std::{io, thread};
use std::io::{ErrorKind, Read};
use std::mem::size_of;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use protocol::packet::{CwSerializable, Packet, PacketId};
use protocol::packet::chat_message::{ChatMessageFromClient, ChatMessageFromServer};
use protocol::packet::chunk_discovery::ChunkDiscovery;
use protocol::packet::creature_action::{CreatureAction, CreatureActionType};
use protocol::packet::creature_update::{Affiliation, CreatureId, CreatureUpdate};
use protocol::packet::hit::Hit;
use protocol::packet::map_seed::MapSeed;
use protocol::packet::player_initialization::PlayerInitialization;
use protocol::packet::projectile::Projectile;
use protocol::packet::protocol_version::ProtocolVersion;
use protocol::packet::sector_discovery::SectorDiscovery;
use protocol::packet::status_effect::StatusEffect;
use protocol::packet::world_update::WorldUpdate;
use protocol::utils::{ReadExtension, WriteExtension};
use crate::creature_id_pool::CreatureIdPool;
use crate::player::Player;

pub struct Server {
	players: RwLock<Vec<Arc<Player>>>,
	id_pool: RwLock<CreatureIdPool>
}

impl Server {
	pub fn new() -> Self {
		Self {
			players: RwLock::new(Vec::new()),
			id_pool: RwLock::new(CreatureIdPool::new())
		}
	}

	pub fn run(self) {
		{ self.id_pool.write().unwrap().claim(); } //reserve 0 for the server itself

		let listener = TcpListener::bind("0.0.0.0:12345").expect("unable to bind listening socket");

		let arc = Arc::new(self);

		loop {
			let (mut stream, _) = listener.accept().unwrap();
			let arc2 = arc.clone();
			thread::spawn(move || {
				if let Err(_) = handle_new_connection(arc2, &mut stream) {
					//TODO: error logging
				}
				stream.shutdown(Shutdown::Both).expect("TODO: panic message");
			});
		}
	}

	pub fn broadcast<P: Packet>(&self, packet: &P, player_to_skip: Option<&Arc<Player>>) where [(); size_of::<P>()]: {
		for player in self.players.read().unwrap().iter() {
			if match player_to_skip {
				Some(player_to_skip) => Arc::ptr_eq(player, player_to_skip),
				None => false
			} { continue }
			player.send(packet);
		}
	}
}

fn handle_new_connection(server: Arc<Server>, stream: &mut TcpStream) -> Result<(), io::Error> {
	if stream.read_struct::<PacketId>()? != PacketId::ProtocolVersion
		|| ProtocolVersion::read_from(stream)?.0 != 3 {
		return Err(io::Error::from(ErrorKind::InvalidData))
	}
	let assigned_id = { server.id_pool.write().unwrap().claim() };
	let result = handle_new_player(&server, stream, assigned_id);
	{ server.id_pool.write().unwrap().free(assigned_id); }
	result
}

fn handle_new_player(server: &Arc<Server>, stream: &mut TcpStream, assigned_id: CreatureId) -> Result<(), io::Error> {
	stream.write_struct(&PacketId::PlayerInitialization)?;
	let player_initialization = PlayerInitialization {
		assigned_id,
		..Default::default()
	};
	player_initialization.write_to(stream)?;

	if stream.read_struct::<PacketId>()? != PacketId::CreatureUpdate {
		return Err(io::Error::from(ErrorKind::InvalidData))
	}
	let full_creature_update = CreatureUpdate::read_from(stream)?;

	let me = Player::new(
		full_creature_update,
		stream,
	);

	me.send(&MapSeed(225));
	me.send(&ChatMessageFromServer {
		source: CreatureId(0),
		text: "welcome to berld".to_string()
	});

	for player in server.players.read().unwrap().iter() {
		me.send(&player.creature);
	}

	let player_arc = Arc::new(me);
	{ server.players.write().unwrap().push(player_arc.clone()); };
	server.broadcast(&player_arc.creature, None);

	read_packets(server, player_arc.clone(), stream).expect_err("impossible");

	{
		let mut players = server.players.write().unwrap();
		let index = players.iter().position(|it| { Arc::ptr_eq(&player_arc, it) }).expect("player not found");
		players.swap_remove(index);
	};
	server.broadcast(&CreatureUpdate {
		id: assigned_id,
		health: Some(0f32),
		affiliation: Some(Affiliation::Neutral),
		..Default::default()
	}, None);

	Ok(())
}

fn read_packets<T: Read>(server: &Arc<Server>, source: Arc<Player>, readable: &mut T) -> Result<(), io::Error> {
	loop {
		let packet_id = readable.read_struct::<PacketId>()?;
		match packet_id {
			PacketId::CreatureUpdate => {
				let creature_update = CreatureUpdate::read_from(readable)?;

				server.broadcast(&creature_update, Some(&source));
			},
			PacketId::CreatureAction => {
				let creature_action = CreatureAction::read_from(readable)?;
				match creature_action.type_ {
					CreatureActionType::Bomb => {}
					CreatureActionType::Talk => {}
					CreatureActionType::ObjectInteraction => {}
					CreatureActionType::PickUp => {}
					CreatureActionType::Drop => {}
					CreatureActionType::CallPet => {}
				}
			}
			PacketId::Hit => {
				let hit = Hit::read_from(readable)?;
				server.broadcast(&hit, Some(&source));
			},
			PacketId::StatusEffect => {
				let status_effect = StatusEffect::read_from(readable)?;
				server.broadcast(
					&WorldUpdate {
						status_effects: vec![status_effect],
						..Default::default()
					},
					Some(&source)
				);
			}
			PacketId::Projectile => {
				let projectile = Projectile::read_from(readable)?;
				server.broadcast(
					&WorldUpdate {
						projectiles: vec![projectile],
						..Default::default()
					},
					Some(&source)
				);
			}
			PacketId::ChatMessage => {
				let chat_message = ChatMessageFromClient::read_from(readable)?;
				server.broadcast(
					&ChatMessageFromServer {
						source: source.creature.id,
						text: chat_message.text
					},
					None
				);
			}
			PacketId::ChunkDiscovery => {
				let _ = ChunkDiscovery::read_from(readable)?;
			}
			PacketId::SectorDiscovery => {
				let _ = SectorDiscovery::read_from(readable)?;
			}
			_ => { panic!("unexpected packet id {:?}", packet_id); }
		}
	}
}