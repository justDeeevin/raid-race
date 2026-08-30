use bevy::{platform::cell::SyncCell, prelude::*};
use clap::{Args, Parser, error::ErrorKind};
use raid_race_lib::{
    component::alive::{
        Cdr, Health, Id,
        player::{
            character::{Character, CharacterData, CharacterName, Cooldowns},
            weapon::{HeldWeapon, Weapon},
        },
        status::Poison,
    },
    system::player::character::warrior,
};
use rustyline::{DefaultEditor, config::Configurer, error::ReadlineError};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use tracing::{error, instrument};

#[derive(Parser)]
#[command(help_template = "{subcommands}")]
enum Command {
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
    /// Quit
    Quit {
        #[arg(default_value_t)]
        /// The exit code
        code: u8,
    },
}

#[derive(Event, Args)]
struct HealthCommand {
    #[arg()]
    /// The id of the entity to choose
    target: u64,
    #[arg(value_parser = |s: &str| if s == "max" {Ok(HealthInput::Max)} else {s.parse().map(HealthInput::Amount)})]
    /// The target health
    ///
    /// Either a number or "max"
    amount: HealthInput,
}

#[derive(Clone)]
enum HealthInput {
    Max,
    Amount(u16),
}

fn health(event: On<HealthCommand>, mut healths: Query<(&Id, &mut Health)>) {
    if let Some(mut target) = healths.iter_mut().find_map(|(id, health)| {
        if **id == event.target {
            Some(health)
        } else {
            None
        }
    }) {
        target.current = match event.amount {
            HealthInput::Amount(amount) => amount,
            HealthInput::Max => target.cap,
        };
    } else {
        error!("target not found");
    }
}

#[derive(Event, Args)]
struct WeaponCommand {
    #[arg()]
    /// The id of the entity to choose
    target: u64,
    #[arg(value_enum)]
    /// The weapon to choose
    weapon: Weapon,
}

#[instrument(skip_all)]
fn weapon(event: On<WeaponCommand>, players: Query<(&Id, Entity)>, mut commands: Commands) {
    if let Some(player) = players.iter().find_map(|(id, entity)| {
        if **id == event.target {
            Some(entity)
        } else {
            None
        }
    }) {
        commands.entity(player).insert(HeldWeapon(event.weapon));
    } else {
        error!("target not found");
    }
}

#[derive(Event, Args)]
struct CharacterCommand {
    #[arg()]
    /// The id of the entity to choose
    target: u64,
    #[arg(value_enum)]
    /// The name of the character to choose
    character: CharacterName,
}

#[instrument(skip_all)]
fn character(event: On<CharacterCommand>, players: Query<(&Id, Entity)>, mut commands: Commands) {
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

    match event.character {
        CharacterName::Warrior => {
            let (character, abilities) = Character::warrior(10);
            commands
                .entity(player)
                .insert((character, Cooldowns::from(&abilities)));
        }
    }
}

#[derive(Event, Args)]
struct SlotCommand {
    #[arg()]
    /// The id of the entity to slot
    target: u64,
    #[arg()]
    /// The name of the ability to slot
    ability: String,
    #[arg()]
    /// The slot to fill
    slot: usize,
}

#[instrument(skip_all)]
fn slot(event: On<SlotCommand>, mut warriors: Query<(&Id, &mut Character)>) {
    let Some(mut character) = warriors.iter_mut().find_map(|(id, character)| {
        if **id == event.target {
            Some(character)
        } else {
            None
        }
    }) else {
        error!("target not found");
        return;
    };

    match &mut character.data {
        CharacterData::Warrior {
            abilities,
            combo_slot,
            ..
        } => {
            let Some(slot) = abilities.get_mut(event.slot - 1) else {
                error!("invalid slot");
                return;
            };

            if let Ok(ability) = event.ability.parse::<warrior::AbilityId>() {
                *slot = ability;
                if ability == warrior::AbilityId::StrikeCombo {
                    *combo_slot = Some(event.slot);
                }
            }
        }
    }
}

#[derive(Event, Args)]
struct PoisonCommand {
    #[arg()]
    /// The id of the entity to poison
    target: u64,
    #[arg()]
    /// The id of the source of the poison
    source: u64,
    #[arg(value_parser = |s: &str| s.parse::<f32>().map(Duration::from_secs_f32))]
    /// The duration of the poison in seconds (decimals accepted)
    duration: Duration,
}

#[instrument(skip_all)]
fn poison(
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

fn thread(tx: Sender<Command>) -> impl FnOnce() {
    move || {
        std::thread::sleep(Duration::from_millis(100));
        let mut rl = DefaultEditor::new().expect("failed to create readline");
        rl.set_auto_add_history(true);

        loop {
            match rl.readline("$ ") {
                Ok(line) => {
                    if !line.trim().is_empty() {
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
                Err(ReadlineError::Interrupted | ReadlineError::Signal(_)) => {}
                Err(error) => {
                    if !matches!(error, ReadlineError::Eof) {
                        error!("{error}");
                    }
                    tx.send(Command::Quit {
                        code: match error {
                            ReadlineError::Io(e) => e.raw_os_error().unwrap_or(1) as u8,
                            ReadlineError::Eof => 0,
                            ReadlineError::Errno(e) => e as u8,
                            _ => 1,
                        },
                    })
                    .expect("failed to send quit command over channel");
                    break;
                }
            }
        }
    }
}

fn handle(mut rx: SyncCell<Receiver<Command>>, app: &mut App) {
    app.add_systems(
        Update,
        move |mut commands: Commands, mut writer: MessageWriter<AppExit>| match rx.get().try_recv()
        {
            Ok(Command::Poison(cmd)) => commands.trigger(cmd),
            Ok(Command::Slot(cmd)) => commands.trigger(cmd),
            Ok(Command::Character(cmd)) => commands.trigger(cmd),
            Ok(Command::Weapon(cmd)) => commands.trigger(cmd),
            Ok(Command::Health(cmd)) => commands.trigger(cmd),
            Ok(Command::Quit { code }) => {
                writer.write(AppExit::from_code(code));
            }
            Err(_) => {}
        },
    );
}

pub fn plugin(app: &mut App) {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(thread(tx));

    handle(SyncCell::new(rx), app);

    app.add_observer(poison)
        .add_observer(slot)
        .add_observer(weapon)
        .add_observer(health)
        .add_observer(character);
}
