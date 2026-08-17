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

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<PanOrbitCam>();
}
