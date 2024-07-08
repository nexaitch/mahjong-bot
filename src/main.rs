#![warn(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]

pub mod strategies;
pub mod utils;
pub mod agent;
pub mod play_mahjong;
pub mod player_information;
pub mod game_logic;

use std::env;

use crate::agent::*;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    let output = play_mahjong::play_mahjong(&mut [Box::new(VeryStupid), Box::new(VeryStupid), Box::new(VeryStupid), Box::new(VeryStupid)]);
    println!("{:?}", output.final_points);
}
