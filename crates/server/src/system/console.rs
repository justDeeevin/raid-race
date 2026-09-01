use bevy::{platform::cell::SyncCell, prelude::*};
use clap::{ArgAction, Args, Parser, error::ErrorKind};
use lightyear::{
    connection::network_target::Target, link::server::Server, prelude::ServerMultiMessageSender,
};
use raid_race_lib::{
    Channel,
    component::alive::{
        Cdr, Health, Id,
        player::{
            character::{Character, CharacterData, CharacterName, Cooldowns},
            weapon::{HeldWeapon, Weapon},
        },
        status::Poison,
    },
    event::{NoCD, Slotted},
    system::player::character::warrior,
};
use rustyline::{DefaultEditor, config::Configurer, error::ReadlineError};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use tracing::{error, instrument};

#[derive(Parser)]
#[command(help_template = "{subcommands}", rename_all = "lowercase")]
enum Command {
    /// Apply poison.
    Poison(PoisonCommand),
    /// Choose a character.
    Character(CharacterCommand),
    /// Slot an ability.
    Slot(SlotCommand),
    /// Equip a weapon.
    Weapon(WeaponCommand),
    /// Set health.
    Health(HealthCommand),
    /// Quit.
    Quit {
        #[arg(default_value_t)]
        /// The exit code.
        code: u8,
    },
    /// Manage no-cooldown mode.
    NoCD {
        #[arg(action = ArgAction::Set)]
        /// The new value.
        enable: Option<bool>,
    },
}

fn no_cd(
    event: On<NoCD>,
    mut no_cd: ResMut<NoCD>,
    mut tx: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    *no_cd = *event;
    #[allow(
        clippy::unwrap_used,
        reason = "should never fail because channel is reliable"
    )]
    tx.send::<_, Channel>(&*event, &server, &Target::All)
        .unwrap();
}

#[derive(Event, Args)]
struct HealthCommand {
    #[arg()]
    /// The id of the entity to choose.
    target: u64,
    #[arg(value_parser = |s: &str| if s == "max" {Ok(HealthInput::Max)} else {s.parse().map(HealthInput::Amount)})]
    /// The new value.
    ///
    /// Either a number or "max".
    amount: Option<HealthInput>,
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
        match &event.amount {
            Some(amount) => {
                target.current = match amount {
                    HealthInput::Amount(n) => *n,
                    HealthInput::Max => target.cap,
                }
            }
            None => info!(health = target.current),
        }
    } else {
        error!("target not found. does the entity have health?");
    }
}

#[derive(Event, Args)]
struct WeaponCommand {
    #[arg()]
    /// The id of the entity to choose.
    target: u64,
    #[arg(value_enum)]
    /// The weapon to choose.
    weapon: Option<Weapon>,
}

#[instrument(skip_all)]
fn weapon(
    event: On<WeaponCommand>,
    players: Query<(&Id, Entity, Option<&HeldWeapon>)>,
    mut commands: Commands,
) {
    if let Some((player, weapon)) = players.iter().find_map(|(id, entity, weapon)| {
        if **id == event.target {
            Some((entity, weapon))
        } else {
            None
        }
    }) {
        match &event.weapon {
            Some(weapon) => {
                commands.entity(player).insert(HeldWeapon(*weapon));
            }
            None => {
                tracing::info!(weapon = %weapon.map(|w| w.0.into()).unwrap_or("None"))
            }
        }
    } else {
        error!("target not found");
    }
}

#[derive(Event, Args)]
struct CharacterCommand {
    #[arg()]
    /// The id of the entity to choose.
    target: u64,
    #[arg(value_enum)]
    /// The name of the character to choose.
    character: Option<CharacterName>,
}

#[instrument(skip_all)]
fn character(
    event: On<CharacterCommand>,
    players: Query<(&Id, Entity, Option<&Character>)>,
    mut commands: Commands,
) {
    let Some((player, character)) = players.iter().find_map(|(id, entity, character)| {
        if **id == event.target {
            Some((entity, character))
        } else {
            None
        }
    }) else {
        error!("target not found");
        return;
    };

    match event.character {
        Some(CharacterName::Warrior) => {
            let (character, abilities) = Character::warrior(10);
            commands
                .entity(player)
                .insert((character, Cooldowns::from(&abilities)));
        }
        None => info!(character = %character.map(|c| (&c.data).into()).unwrap_or("None"),),
    }
}

#[derive(Event, Args)]
struct SlotCommand {
    #[arg()]
    /// The id of the entity to choose.
    target: u64,
    #[arg()]
    /// The slot to fill.
    slot: usize,
    /// The name of the ability to slot.
    ability: Option<String>,
}

#[instrument(skip_all)]
fn slot(
    event: On<SlotCommand>,
    mut warriors: Query<(&Id, Entity, &mut Character)>,
    mut tx: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    let Some((entity, mut character)) = warriors.iter_mut().find_map(|(id, entity, character)| {
        if **id == event.target {
            Some((entity, character))
        } else {
            None
        }
    }) else {
        error!("target not found. does the entity have an assigned character?");
        return;
    };

    let Some(ability) = &event.ability else {
        info!(ability = %character.data.ability(event.slot));
        return;
    };

    match &mut character.data {
        CharacterData::Warrior {
            abilities,
            combo_index,
            ..
        } => {
            let Some(slot) = abilities.get_mut(event.slot - 1) else {
                error!("invalid slot");
                return;
            };

            if let Ok(ability) = ability.parse::<warrior::AbilityId>() {
                *slot = ability;
                if ability == warrior::AbilityId::StrikeCombo {
                    *combo_index = Some(event.slot - 1);
                }
            } else {
                error!("invalid ability");
            }
        }
    }

    #[allow(
        clippy::unwrap_used,
        reason = "should never fail because channel is reliable"
    )]
    tx.send::<_, Channel>(
        &Slotted {
            entity,
            index: event.slot - 1,
        },
        &server,
        &Target::All,
    )
    .unwrap();
}

#[derive(Event, Args)]
struct PoisonCommand {
    #[arg()]
    /// The id of the entity to choose.
    target: u64,
    #[arg()]
    /// The id of the source of the poison.
    source: u64,
    #[arg(value_parser = |s: &str| s.parse::<f32>().map(Duration::from_secs_f32))]
    /// The duration of the poison in seconds (decimals accepted).
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
        let mut rl = DefaultEditor::new().expect("failed to create readline");
        rl.set_auto_add_history(true);

        loop {
            std::thread::sleep(Duration::from_millis(100));
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
        move |mut commands: Commands, mut writer: MessageWriter<AppExit>, no_cd: Res<NoCD>| match rx
            .get()
            .try_recv()
        {
            Ok(Command::Poison(cmd)) => commands.trigger(cmd),
            Ok(Command::Slot(cmd)) => commands.trigger(cmd),
            Ok(Command::Character(cmd)) => commands.trigger(cmd),
            Ok(Command::Weapon(cmd)) => commands.trigger(cmd),
            Ok(Command::Health(cmd)) => commands.trigger(cmd),
            Ok(Command::Quit { code }) => {
                writer.write(AppExit::from_code(code));
            }
            Ok(Command::NoCD { enable }) => match enable {
                Some(enable) => commands.trigger(NoCD(enable)),
                None => info!(no_cd = **no_cd),
            },
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
        .add_observer(character)
        .add_observer(no_cd);
}
