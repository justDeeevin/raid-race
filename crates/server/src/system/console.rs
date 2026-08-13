use bevy::{
    app::{App, Update},
    ecs::{
        entity::Entity,
        event::Event,
        observer::On,
        system::{Commands, Query},
    },
    platform::cell::SyncCell,
};
use clap::{Args, Parser, error::ErrorKind};
use raid_race_lib::component::alive::{Cdr, Id, status::Poison};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use tracing::error;

#[derive(Parser)]
#[command(help_template = "{subcommands}")]
pub enum Command {
    /// Apply poison to something
    Poison(PoisonCommand),
}

#[derive(Event, Args)]
pub struct PoisonCommand {
    #[arg()]
    /// The id of the entity to poison
    pub target: u64,
    #[arg()]
    /// The id of the source of the poison
    pub source: u64,
    #[arg(value_parser = |s: &str| s.parse::<f32>().map(Duration::from_secs_f32))]
    /// The duration of the poison in seconds (decimals accepted)
    pub duration: Duration,
}

pub fn poison(
    cmd: On<PoisonCommand>,
    mut commands: Commands,
    entities: Query<(Entity, &Id, Option<&Cdr>)>,
) {
    let Some(target) = entities.iter().find_map(|(entity, id, _)| {
        if **id == cmd.target {
            Some(entity)
        } else {
            None
        }
    }) else {
        error!("poison target not found");
        return;
    };

    let Some((source, cdr)) = entities.iter().find_map(|(entity, id, cdr)| {
        if **id == cmd.source
            && let Some(cdr) = cdr
        {
            Some((entity, cdr))
        } else {
            None
        }
    }) else {
        error!("poison source not found");
        return;
    };

    commands
        .entity(target)
        .insert(Poison::new(source, cdr, cmd.duration));
}

pub fn thread(tx: Sender<Command>) -> impl FnOnce() {
    move || {
        for line in std::io::stdin().lines().map(Result::unwrap) {
            match Command::try_parse_from([""].into_iter().chain(line.split(' '))) {
                Ok(cmd) => tx.send(cmd).expect("failed to send command over channel"),
                Err(error) => match error.kind() {
                    ErrorKind::DisplayHelp => {
                        println!("{error}");
                    }
                    _ => {
                        error!("{error}");
                    }
                },
            }
        }
    }
}

pub fn handle(mut rx: SyncCell<Receiver<Command>>, app: &mut App) {
    app.add_systems(Update, move |mut commands: Commands| {
        match rx.get().try_recv() {
            Ok(Command::Poison(cmd)) => commands.trigger(cmd),
            Err(_) => {}
        }
    });
}
