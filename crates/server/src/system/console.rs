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
use raid_race_lib::{
    component::alive::{Cdr, Health, Id, status::Poison},
    player::{
        character::{
            Abilities, Cooldowns,
            warrior::{self, Warrior},
        },
        weapon::placeholder_gun::PlaceholderGun,
    },
};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use tracing::{error, instrument};

#[derive(Parser)]
#[command(help_template = "{subcommands}")]
pub enum Command {
    /// Apply poison
    Poison(PoisonCommand),
    /// Choose a character
    Character(CharacterCommand),
    /// Slot an ability
    Slot(SlotCommand),
    /// Equip a weapon
    Weapon(WeaponCommand),
    /// Set health
    Health(HealthCommand),
}

#[derive(Event, Args)]
pub struct HealthCommand {
    #[arg()]
    /// The id of the entity to choose
    pub target: u64,
    #[arg(value_parser = |s: &str| if s == "max" {Ok(HealthInput::Max)} else {s.parse().map(HealthInput::Amount)})]
    /// The target health
    ///
    /// Either a number or "max"
    pub amount: HealthInput,
}

#[derive(Clone)]
pub enum HealthInput {
    Max,
    Amount(u16),
}

pub fn health(event: On<HealthCommand>, mut healths: Query<(&Id, &mut Health)>) {
    let Some(mut target) = healths.iter_mut().find_map(|(id, health)| {
        if **id == event.target {
            Some(health)
        } else {
            None
        }
    }) else {
        error!("target not found");
        return;
    };

    target.current = match event.amount {
        HealthInput::Amount(amount) => amount,
        HealthInput::Max => target.cap,
    };
}

#[derive(Event, Args)]
pub struct WeaponCommand {
    #[arg()]
    /// The id of the entity to choose
    pub target: u64,
    #[arg()]
    /// The name of the weapon to choose
    pub weapon: String,
}

#[instrument(skip_all)]
pub fn weapon(event: On<WeaponCommand>, players: Query<(&Id, Entity)>, mut commands: Commands) {
    let Some(player) = players.iter().find_map(|(id, entity)| {
        if **id == event.target {
            Some(entity)
        } else {
            None
        }
    }) else {
        error!("target not found");
        return;
    };

    match event.weapon.as_str() {
        "placeholder" => {
            commands.entity(player).insert(PlaceholderGun);
        }
        _ => error!("unknown weapon"),
    }
}

#[derive(Event, Args)]
pub struct CharacterCommand {
    #[arg()]
    /// The id of the entity to choose
    pub target: u64,
    #[arg()]
    /// The name of the character to choose
    pub character: String,
}

#[instrument(skip_all)]
pub fn character(
    event: On<CharacterCommand>,
    players: Query<(&Id, Entity)>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().find_map(|(id, entity)| {
        if **id == event.target {
            Some(entity)
        } else {
            None
        }
    }) else {
        error!("target not found");
        return;
    };

    match event.character.as_str() {
        "warrior" => {
            let abilities = Abilities::<warrior::AbilityId>::default();
            commands.entity(player).insert((
                Cooldowns::from(&abilities),
                abilities,
                Warrior::new(10),
            ));
        }
        _ => error!("unknown character"),
    }
}

#[derive(Event, Args)]
pub struct SlotCommand {
    #[arg()]
    /// The id of the entity to slot
    pub target: u64,
    #[arg()]
    /// The name of the ability to slot
    pub ability: String,
    #[arg()]
    /// The slot to fill
    pub slot: u8,
}

#[instrument(skip_all)]
pub fn slot(
    event: On<SlotCommand>,
    mut warriors: Query<(&Id, &mut Abilities<warrior::AbilityId>)>,
) {
    if let Some(mut abilities) = warriors.iter_mut().find_map(|(id, abilities)| {
        if **id == event.target {
            Some(abilities)
        } else {
            None
        }
    }) {
        let id = match event.ability.as_str() {
            "strike" => warrior::AbilityId::Strike,
            "combo" => warrior::AbilityId::StrikeCombo,
            _ => {
                error!("unknown ability for warrior");
                return;
            }
        };
        if let Some(ability) = abilities.get_mut(event.slot as usize - 1) {
            *ability = id;
        } else {
            error!("invalid slot");
        }
    } else {
        error!("target not found")
    }
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

#[instrument(skip_all)]
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
        error!("target not found");
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
        error!("source not found");
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
            Ok(Command::Slot(cmd)) => commands.trigger(cmd),
            Ok(Command::Character(cmd)) => commands.trigger(cmd),
            Ok(Command::Weapon(cmd)) => commands.trigger(cmd),
            Ok(Command::Health(cmd)) => commands.trigger(cmd),
            Err(_) => {}
        }
    });
}
