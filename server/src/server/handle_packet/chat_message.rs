use std::sync::atomic::Ordering::Relaxed;

use colour::{cyan, white_ln};

use protocol::packet::ChatMessageFromClient;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<ChatMessageFromClient> for Server {
	async fn handle_packet(&self, source: &Player, packet: ChatMessageFromClient) {
		let character_guard = source.character.read().await;
		cyan!("{}: ", character_guard.name);
		white_ln!("{}", packet.text);

		if self.addons.command_manager.on_message(self, Some(source), source.admin.load(Relaxed), &packet.text, '/', |message| { source.notify(message) }).await {
			return;
		}

		self.addons.discord_integration.post(&format!("**{}:** {}", character_guard.name, packet.text), false).await;

		self.broadcast(&packet.into_reverse(source.id), None).await;
		self.play_chat_sound().await;
	}
}