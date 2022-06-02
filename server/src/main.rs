#![feature(generic_const_exprs)]

use server::Server;

mod server;
mod creature_id_pool;
mod player;
mod traffic_filter;
mod pvp;

fn main() {
	println!("go");
	Server::new().run();
}

//todo
//nothing type?
//half open connection vulnerability
//lock scoping necessary?