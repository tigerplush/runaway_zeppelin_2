use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
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

pub fn pan_orbit_controls() -> impl Bundle {
    (
        PanOrbitCam,
        actions!(PanOrbitCam[
            (
                Action::<Pan>::new(),
                DeadZone::default(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
            ),
            (
                Action::<Orbit>::new(),
                DeadZone::default(),
                Bindings::spawn((
                    Spawn(Binding::mouse_motion()),
                    Axial::right_stick()
                ))
            ),
            (
                Action::<Zoom>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ, Negate::all())),
                    Bidirectional::new(GamepadButton::DPadUp, GamepadButton::DPadDown),
                ))
            ),
            (
                Action::<EnableOrbit>::new(),
                Bindings::spawn((
                    Spawn(Binding::from(MouseButton::Middle)),
                    Axial::right_stick().with(DeadZone::default()),
                )),
            )
        ]),
    )
}


#[derive(Component)]
pub struct TimeControl;

#[derive(InputAction)]
#[action_output(bool)]
pub struct SetSpeed;

#[derive(Clone, Copy, PartialEq, Reflect)]
pub enum GameSpeed {
    Pause,
    Speedx1,
    Speedx2,
    Speedx4
}

#[derive(Component, Deref)]
pub struct GameSpeedIndex(pub GameSpeed);

pub fn time_control() -> impl Bundle {
    (
        TimeControl,
        actions!(TimeControl[
            (Action::<SetSpeed>::new(), GameSpeedIndex(GameSpeed::Pause), bindings![KeyCode::Space]),
            (Action::<SetSpeed>::new(), GameSpeedIndex(GameSpeed::Speedx1), bindings![KeyCode::Digit1]),
            (Action::<SetSpeed>::new(), GameSpeedIndex(GameSpeed::Speedx2), bindings![KeyCode::Digit2]),
            (Action::<SetSpeed>::new(), GameSpeedIndex(GameSpeed::Speedx4), bindings![KeyCode::Digit3]),
        ]),
    )
}

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<PanOrbitCam>()
        .add_input_context::<TimeControl>();
}
