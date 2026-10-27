use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Clone, Component, Default)]
pub struct PanOrbitCam;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Pan;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Orbit;

#[derive(InputAction)]
#[action_output(f32)]
pub struct Zoom;

#[derive(InputAction)]
#[action_output(bool)]
pub struct EnableOrbit;

pub fn pan_orbit_controls_bsn() -> impl Scene {
    bsn! {
        PanOrbitCam
        Actions<PanOrbitCam> [
            Action::<Pan>::new()
            DeadZone::default()
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
            --
            Action::<Orbit>::new()
            DeadZone::default()
            Bindings [
                Binding::mouse_motion()
                --
                {preset::axial::right_stick()}
            ]
            --
            Action::<Zoom>::new()
            Bindings [
                Binding::mouse_wheel()
                ~SwizzleAxis::YXZ
                ~Negate::all()
                --
                {preset::bidirectional::new(GamepadButton::DPadUp, GamepadButton::DPadDown)}
            ]
            --
            Action::<EnableOrbit>::new()
            Bindings [
                Binding(MouseButton::Middle)
                --
                {preset::axial::right_stick()}
            ]
        ]
    }
}

#[derive(Clone, Component, Default)]
pub struct TimeControl;

#[derive(Clone, Default, InputAction)]
#[action_output(bool)]
pub enum SetSpeed {
    TogglePause,
    #[default]
    Speedx1,
    Speedx2,
    Speedx4,
}

#[derive(InputAction)]
#[action_output(bool)]
pub struct AccelerateTimeDilation;

#[derive(Clone, Component, Default, Deref)]
pub struct GameSpeedIndex(pub SetSpeed);

pub fn time_control_bsn() -> impl Scene {
    bsn! {
        TimeControl
        Actions<TimeControl> [
            Action::<SetSpeed>::new()
            GameSpeedIndex(SetSpeed::TogglePause)
            Bindings [
                Binding(KeyCode::Space)
            ]
            --
            Action::<SetSpeed>::new()
            GameSpeedIndex(SetSpeed::Speedx1)
            Bindings [
                Binding(KeyCode::Digit1)
            ]
            --
            Action::<SetSpeed>::new()
            GameSpeedIndex(SetSpeed::Speedx2)
            Bindings [
                Binding(KeyCode::Digit2)
            ]
            --
            Action::<SetSpeed>::new()
            GameSpeedIndex(SetSpeed::Speedx4)
            Bindings [
                Binding(KeyCode::Digit3)
            ]
            --
            Action::<AccelerateTimeDilation>::new()
            Bindings [
                Binding(KeyCode::ShiftLeft)
            ]
        ]
    }
}

#[derive(Clone, Component, Default)]
pub struct PointerControl;

#[derive(InputAction)]
#[action_output(bool)]
pub struct SelectTile;

pub fn pointer_control_bsn() -> impl Scene {
    bsn! {
        PointerControl
        Actions<PointerControl> [
            Action::<SelectTile>::new()
            Bindings [
                Binding(MouseButton::Left)
            ]
        ]
    }
}

#[derive(Component)]
struct BindingsMap;

fn setup(mut commands: Commands) {
    commands.spawn((Name::from("Bindings"), BindingsMap));
}

fn sort_bindings(
    trigger: On<Add<Binding>>,
    bindings: Single<Entity, With<BindingsMap>>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .insert(ChildOf(bindings.entity()));
}

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<PanOrbitCam>()
        .add_input_context::<TimeControl>()
        .add_input_context::<PointerControl>()
        .add_systems(Startup, setup)
        .add_observer(sort_bindings);
}
