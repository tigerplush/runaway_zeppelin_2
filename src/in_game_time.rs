use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use chrono::*;

use crate::{
    input::*,
    states::AppStates,
    ui::{self, AttachToUiSlot},
};

const STARTING_DATE: &str = "1928-10-11 08:00:00";
const FROM_STRING_FMT: &str = "%Y-%m-%d %H:%M:%S";
const TO_STRING_FMT: &str = "%A, %_H:%M %_d %B %Y";
const DEFAULT_TIME_SPEED_FACTOR: f32 = 48.0;

const ACCELERATED_TIME_DILATION: f32 = 5.0;

#[derive(Clone, Copy, PartialEq, Reflect)]
enum GameSpeed {
    Pause,
    Speedx1,
    Speedx2,
    Speedx4,
}

impl GameSpeed {
    fn as_f32(&self) -> f32 {
        match self {
            GameSpeed::Pause => 0f32,
            GameSpeed::Speedx1 => 1f32,
            GameSpeed::Speedx2 => 2f32,
            GameSpeed::Speedx4 => 4f32,
        }
    }
}

/// Represents the in-game time of a singular run.
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct InGameTime {
    #[reflect(ignore)]
    current_time: NaiveDateTime,
    #[reflect(ignore)]
    starting_time: NaiveDateTime,
    current_speed: GameSpeed,
    previous_speed: GameSpeed,
}

impl InGameTime {
    /// Creates a new InGameTime from the given date and time.
    pub fn new(date: NaiveDate, time: NaiveTime) -> Self {
        let date_time = NaiveDateTime::new(date, time);
        Self {
            current_time: date_time,
            starting_time: date_time,
            current_speed: GameSpeed::Pause,
            previous_speed: GameSpeed::Speedx1,
        }
    }

    /// Returns the number of days that have passed in game.
    ///
    /// Note: directly at start, 0 days have passed.
    pub fn days(&self) -> i64 {
        let diff = self.current_time - self.starting_time;
        diff.num_days()
    }

    fn get_current_time_speed(&self) -> f32 {
        self.current_speed.as_f32() * DEFAULT_TIME_SPEED_FACTOR
    }

    pub fn pause(&mut self) {
        if GameSpeed::Pause == self.current_speed {
            return;
        }
        self.previous_speed = self.current_speed;
        self.current_speed = GameSpeed::Pause
    }

    pub fn unpause(&mut self) {
        if GameSpeed::Pause != self.current_speed {
            return;
        }
        self.current_speed = self.previous_speed
    }

    fn toggle_pause(&mut self) {
        if GameSpeed::Pause == self.current_speed {
            self.unpause();
        } else {
            self.pause();
        }
    }
}

impl Default for InGameTime {
    fn default() -> Self {
        let date = NaiveDate::parse_from_str(STARTING_DATE, FROM_STRING_FMT);
        let time = NaiveTime::parse_from_str(STARTING_DATE, FROM_STRING_FMT);
        Self::new(date.unwrap(), time.unwrap())
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

    in_game_time.current_time += time.delta();
}

#[derive(Component)]
struct InGameTimeLabel;

fn setup(in_game_time: Res<InGameTime>, mut commands: Commands) {
    commands.spawn((Name::from("Time Controls"), time_control()));
    commands.spawn((
        ui::label(
            format!("{}", *in_game_time),
            AttachToUiSlot::InGameTimeLabel,
        ),
        InGameTimeLabel,
    ));
}

fn update_label(
    in_game_time: Res<InGameTime>,
    mut label: Single<&mut Text, With<InGameTimeLabel>>,
) {
    label.0 = format!("{}", *in_game_time);
}

fn on_set_speed(
    trigger: On<Start<SetSpeed>>,
    mut in_game_time: ResMut<InGameTime>,
    actions: Query<&GameSpeedIndex>,
) {
    let Ok(game_speed) = actions.get(trigger.action) else {
        return;
    };
    match game_speed.0 {
        SetSpeed::TogglePause => in_game_time.toggle_pause(),
        SetSpeed::Speedx1 => in_game_time.current_speed = GameSpeed::Speedx1,
        SetSpeed::Speedx2 => in_game_time.current_speed = GameSpeed::Speedx2,
        SetSpeed::Speedx4 => in_game_time.current_speed = GameSpeed::Speedx4,
    };
}

fn apply_speed(mut time: ResMut<Time<Virtual>>, in_game_time: Res<InGameTime>) {
    match in_game_time.current_speed {
        GameSpeed::Pause => time.pause(),
        _ => {
            time.unpause();
            time.set_relative_speed(in_game_time.get_current_time_speed())
        }
    };
}

#[derive(Message)]
pub enum TimeControlMessage {
    Disable,
    Enable,
}

fn on_time_control_message(
    mut reader: MessageReader<TimeControlMessage>,
    single: Single<Entity, With<ContextActivity<TimeControl>>>,
    mut commands: Commands,
) {
    for ev in reader.read() {
        match ev {
            TimeControlMessage::Disable => {
                commands
                    .entity(single.entity())
                    .insert(ContextActivity::<TimeControl>::INACTIVE);
            }
            TimeControlMessage::Enable => {
                commands
                    .entity(single.entity())
                    .insert(ContextActivity::<TimeControl>::ACTIVE);
            }
        }
    }
}

fn on_start_accelerate_time(
    _trigger: On<Start<AccelerateTimeDilation>>,
    in_game_time: Res<InGameTime>,
    mut time: ResMut<Time<Virtual>>,
) {
    time.set_relative_speed(in_game_time.get_current_time_speed() * ACCELERATED_TIME_DILATION);
}

fn on_complete_accelerate_time(
    _trigger: On<Complete<AccelerateTimeDilation>>,
    in_game_time: Res<InGameTime>,
    mut time: ResMut<Time<Virtual>>,
) {
    time.set_relative_speed(in_game_time.get_current_time_speed());
}

pub fn plugin(app: &mut App) {
    app.register_type::<InGameTime>()
        .init_resource::<InGameTime>()
        .add_message::<TimeControlMessage>()
        .add_systems(OnEnter(AppStates::InGame), setup)
        .add_systems(
            Update,
            (
                on_time_control_message,
                tick_in_game_time,
                update_label,
                apply_speed.run_if(resource_changed::<InGameTime>),
            )
                .run_if(resource_exists::<InGameTime>),
        )
        .add_observer(on_set_speed.run_if(resource_exists::<InGameTime>));

    #[cfg(debug_assertions)]
    app.add_observer(on_start_accelerate_time)
        .add_observer(on_complete_accelerate_time);
}
