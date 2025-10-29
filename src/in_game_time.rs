use bevy::prelude::*;
use chrono::{Datelike, NaiveDateTime, Timelike};

use crate::states::AppStates;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppStates::MainApp), (pause_game, init_in_game_time))
        .add_systems(
            Update,
            tick_in_game_time.run_if(in_state(AppStates::MainApp)),
        )
        .add_observer(on_game_speed_change);
}

pub enum GameSpeed {
    Pause,
    Speed1,
    Speed2,
    Speed4,
}

#[derive(Event)]
pub enum TimePassed {
    Minute(u32),
    Hour(u32),
    Day(u32),
}

const TIME_SPEED_FACTOR: f32 = 48.0;

#[derive(Event)]
pub struct GameSpeedChangeEvent(pub GameSpeed);

fn on_game_speed_change(trigger: On<GameSpeedChangeEvent>, mut time: ResMut<Time<Virtual>>) {
    match trigger.0 {
        GameSpeed::Pause => time.set_relative_speed(0.0),
        GameSpeed::Speed1 => time.set_relative_speed(TIME_SPEED_FACTOR),
        GameSpeed::Speed2 => time.set_relative_speed(TIME_SPEED_FACTOR * 2.0),
        GameSpeed::Speed4 => time.set_relative_speed(TIME_SPEED_FACTOR * 4.0),
    }
}

fn pause_game(mut commands: Commands) {
    commands.trigger(GameSpeedChangeEvent(GameSpeed::Pause));
}

#[derive(Reflect, Component)]
#[reflect(Component)]
struct InGameTime {
    #[reflect(ignore)]
    current_time: NaiveDateTime,
    #[reflect(ignore)]
    starting_time: NaiveDateTime,
}

const STARTING_DATE: &str = "1928-10-11 08:00:00";
const FROM_STRING_FMT: &str = "%Y-%m-%d %H:%M:%S";

impl Default for InGameTime {
    fn default() -> Self {
        let date_time = NaiveDateTime::parse_from_str(STARTING_DATE, FROM_STRING_FMT).unwrap();
        InGameTime {
            current_time: date_time,
            starting_time: date_time,
        }
    }
}

fn tick_in_game_time(
    time: Res<Time>,
    mut in_game_time: Single<&mut InGameTime>,
    mut commands: Commands,
) {
    let previous = in_game_time.current_time;
    in_game_time.current_time = in_game_time.current_time + time.delta();
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

fn init_in_game_time(mut commands: Commands) {
    commands.spawn(InGameTime::default());
}
