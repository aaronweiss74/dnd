#![feature(if_let)]
#![feature(slicing_syntax)]
extern crate crypto;
extern crate irc;
extern crate serialize;

use data::world::World;
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

mod data;
mod func;

#[cfg(not(test))]
fn main() {
    let mut world = World::new();
    let server = IrcServer::new("config.json").unwrap();
    for message in server.iter() {
        println!("{}", message.into_string());
        let mut args = Vec::new();
        let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
        args.push_all(msg_args[]);
        if let Some(ref suffix) = message.suffix {
            args.push(suffix[])
        }
        let source = message.prefix.unwrap_or(String::new());
        let mut token_store = Vec::new();
        func::process_world(&Wrapper::new(&server), source[], message.command[], args[],
                            &mut token_store, &mut world).unwrap();
    }
}
