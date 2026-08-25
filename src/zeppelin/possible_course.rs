use std::time::Duration;

use bevy::prelude::*;

use crate::{
    ui::{self, AttachToUiSlot},
    utils::hex::AxialCoordinates,
};

/// Represents a possible course. There will only ever be one, so this is a
/// resource
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub(super) struct PossibleCourse {
    pub(super) target: AxialCoordinates,
    pub(super) path: super::ZeppelinPath,
    pub(super) duration: Duration,
    pub(super) fuel_consumption: f32,
    pub(super) gas_consumption: f32,
}

fn on_set_course(
    _trigger: On<Pointer<Release>>,
    possible_course: Res<PossibleCourse>,
    zeppelin: Single<Entity, With<super::ZeppelinWrapper>>,
    mut commands: Commands,
) {
    commands
        .entity(zeppelin.entity())
        .insert(possible_course.path);
    commands.remove_resource::<PossibleCourse>();
}

#[derive(Component)]
struct SetCourseButton;

fn on_insert_course(
    _trigger: On<Insert, PossibleCourse>,
    possible_course: Res<PossibleCourse>,
    previous_buttons: Query<Entity, With<SetCourseButton>>,
    mut commands: Commands,
) {
    let hours = possible_course.duration.as_secs() / 3600;
    let mins = (possible_course.duration.as_secs()) / 60 % 60;
    let content = format!(
        "Fuel: {:.1}kg | Gas: {:.1}m³ | Time: {}h{}min",
        possible_course.fuel_consumption, possible_course.gas_consumption, hours, mins
    );
    commands
        .spawn((
            ui::primary_button("Set Course", Some(content), AttachToUiSlot::Action),
            SetCourseButton,
        ))
        .observe(on_set_course);

    for entity in &previous_buttons {
        commands.entity(entity).despawn();
    }
}

fn on_remove_course(
    _trigger: On<Remove, PossibleCourse>,
    previous_button: Single<Entity, With<SetCourseButton>>,
    mut commands: Commands,
) {
    commands.entity(previous_button.entity()).despawn();
}

#[cfg(debug_assertions)]
fn debug_course(
    mut gizmos: Gizmos,
    course: Res<PossibleCourse>,
    zeppelin: Single<&Transform, With<super::ZeppelinWrapper>>,
) {
    use bevy::color::palettes::css::ORANGE;

    use crate::utils::hex::DEFAULT_HEX_SIZE;

    let start = zeppelin.translation;
    let end = course.target.as_world_coordinates(DEFAULT_HEX_SIZE);
    gizmos.arrow(start, end, ORANGE);
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PossibleCourse>()
        .add_observer(on_insert_course)
        .add_observer(on_remove_course);
    #[cfg(debug_assertions)]
    app.add_systems(
        Update,
        debug_course.run_if(resource_exists::<PossibleCourse>),
    );
}
