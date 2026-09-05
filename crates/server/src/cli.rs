use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Resource)]
#[command()]
pub struct Args {
    #[arg(short, long, default_value_t)]
    pub visual: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
