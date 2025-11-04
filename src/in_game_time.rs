use std::fmt::Display;

use bevy::prelude::*;
use chrono::{Datelike, NaiveDateTime, Timelike};
use leafwing_input_manager::prelude::*;

use crate::states::AppStates;

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<GameSpeed>::default())
        .add_systems(OnEnter(AppStates::MainApp), (pause_game, init_in_game_time))
        .add_systems(
            Update,
            (tick_in_game_time, check_input).run_if(in_state(AppStates::MainApp)),
        )
        .add_observer(on_game_speed_change);
}

#[derive(Actionlike, Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum GameSpeed {
    Pause,
    Speed1,
    Speed2,
    Speed4,
}
const GAME_SPEEDS: [GameSpeed; 4] = [
    GameSpeed::Pause,
    GameSpeed::Speed1,
    GameSpeed::Speed2,
    GameSpeed::Speed4,
];

#[derive(Event)]
pub enum TimePassed {
    Minute(u32),
    Hour(u32),
    Day(u32),
}

const TIME_SPEED_FACTOR: u32 = 48;

#[derive(Event)]
pub struct GameSpeedChangeEvent(pub GameSpeed);

fn on_game_speed_change(
    trigger: On<GameSpeedChangeEvent>,
    mut time: ResMut<Time<Virtual>>,
    mut in_game_time: Single<&mut InGameTime>,
) {
    let new_speed =
        if GameSpeed::Pause == in_game_time.current_speed && GameSpeed::Pause == trigger.0 {
            in_game_time.current_speed = in_game_time.previous_speed;
            in_game_time.previous_speed
        } else {
            in_game_time.previous_speed = in_game_time.current_speed;
            in_game_time.current_speed = trigger.0;
            in_game_time.current_speed
        };
    match new_speed {
        GameSpeed::Pause => {
            time.pause();
        }
        GameSpeed::Speed1 => {
            time.set_relative_speed(1.0);
            time.unpause();
        }
        GameSpeed::Speed2 => {
            time.set_relative_speed(2.0);
            time.unpause();
        }
        GameSpeed::Speed4 => {
            time.set_relative_speed(4.0);
            time.unpause();
        }
    }
    info!(
        "current: {:?}, previous: {:?}",
        in_game_time.current_speed, in_game_time.previous_speed
    );
}

fn pause_game(mut commands: Commands) {
    commands.trigger(GameSpeedChangeEvent(GameSpeed::Pause));
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct InGameTime {
    #[reflect(ignore)]
    current_time: NaiveDateTime,
    #[reflect(ignore)]
    starting_time: NaiveDateTime,
    previous_speed: GameSpeed,
    current_speed: GameSpeed,
}

const STARTING_DATE: &str = "1928-10-11 08:00:00";
const FROM_STRING_FMT: &str = "%Y-%m-%d %H:%M:%S";
const TO_STRING_FMT: &str = "%A, %_H:%M %_d %B %Y";

impl Default for InGameTime {
    fn default() -> Self {
        let date_time = NaiveDateTime::parse_from_str(STARTING_DATE, FROM_STRING_FMT).unwrap();
        InGameTime {
            current_time: date_time,
            starting_time: date_time,
            previous_speed: GameSpeed::Pause,
            current_speed: GameSpeed::Speed1,
        }
    }
}

impl Display for InGameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (Day {})",
            self.current_time.format(TO_STRING_FMT),
            self.days() + 1
        )
    }
}

impl InGameTime {
    fn days(&self) -> i64 {
        let diff = self.current_time - self.starting_time;
        diff.num_days()
    }
}

fn tick_in_game_time(
    time: Res<Time<Virtual>>,
    mut in_game_time: Single<&mut InGameTime>,
    mut commands: Commands,
) {
    if time.is_paused() {
        return;
    }
    let previous = in_game_time.current_time;
    in_game_time.current_time += time.delta() * TIME_SPEED_FACTOR;
    if in_game_time.current_time.minute() != previous.minute() {
        commands.trigger(TimePassed::Minute(in_game_time.current_time.minute()));
    }
    if in_game_time.current_time.hour() != previous.hour() {
        commands.trigger(TimePassed::Hour(in_game_time.current_time.hour()));
    }
    if in_game_time.current_time.day() != previous.day() {
        commands.trigger(TimePassed::Day(in_game_time.current_time.day()));
    }
}

#[derive(Component)]
struct InGameTimeActions;

fn init_in_game_time(mut commands: Commands) {
    commands.spawn((InGameTime::default(), DespawnOnExit(AppStates::MainMenu)));

    let input_map = InputMap::new([
        (GameSpeed::Pause, KeyCode::Space),
        (GameSpeed::Speed1, KeyCode::Digit1),
        (GameSpeed::Speed2, KeyCode::Digit2),
        (GameSpeed::Speed4, KeyCode::Digit3),
    ]);
    commands.spawn((
        input_map,
        InGameTimeActions,
        DespawnOnExit(AppStates::MainApp),
    ));
}

fn check_input(
    action_map: Single<&ActionState<GameSpeed>, With<InGameTimeActions>>,
    mut commands: Commands,
) {
    for speed in GAME_SPEEDS {
        if action_map.just_pressed(&speed) {
            commands.trigger(GameSpeedChangeEvent(speed));
        }
    }
}
