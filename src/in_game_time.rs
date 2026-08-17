use bevy::prelude::*;
use chrono::*;

const STARTING_DATE: &str = "1928-10-11 08:00:00";
const FROM_STRING_FMT: &str = "%Y-%m-%d %H:%M:%S";
const TO_STRING_FMT: &str = "%A, %_H:%M %_d %B %Y";
const DEFAULT_TIME_SPEED_FACTOR: u32 = 48;

/// Represents the in-game time of a singular run.
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct InGameTime {
    #[reflect(ignore)]
    current_time: NaiveDateTime,
    #[reflect(ignore)]
    starting_time: NaiveDateTime,
    time_speed_factor: u32,
    #[cfg(debug_assertions)]
    debug_string: String,
}

impl InGameTime {
    /// Creates a new InGameTime from the given date and time.
    pub fn new(date: NaiveDate, time: NaiveTime, time_speed_factor: u32) -> Self {
        let date_time = NaiveDateTime::new(date, time);
        Self {
            current_time: date_time,
            starting_time: date_time,
            time_speed_factor,
            #[cfg(debug_assertions)]
            debug_string: String::new(),
        }
    }

    /// Returns the number of days that have passed in game.
    ///
    /// Note: directly at start, 0 days have passed.
    pub fn days(&self) -> i64 {
        let diff = self.current_time - self.starting_time;
        diff.num_days()
    }
}

impl Default for InGameTime {
    fn default() -> Self {
        let date = NaiveDate::parse_from_str(STARTING_DATE, FROM_STRING_FMT);
        let time = NaiveTime::parse_from_str(STARTING_DATE, FROM_STRING_FMT);
        Self::new(date.unwrap(), time.unwrap(), DEFAULT_TIME_SPEED_FACTOR)
    }
}

impl std::fmt::Display for InGameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (Day {})",
            self.current_time.format(TO_STRING_FMT),
            self.days() + 1
        )
    }
}

fn tick_in_game_time(time: Res<Time<Virtual>>, mut in_game_time: ResMut<InGameTime>) {
    if time.is_paused() {
        return;
    }

    let time_speed_factor = in_game_time.time_speed_factor;
    in_game_time.current_time += time.delta() * time_speed_factor;

    if cfg!(debug_assertions) {
        in_game_time.debug_string = format!("{}", *in_game_time);
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<InGameTime>()
        .init_resource::<InGameTime>()
        .add_systems(
            Update,
            tick_in_game_time.run_if(resource_exists::<InGameTime>),
        );
}
