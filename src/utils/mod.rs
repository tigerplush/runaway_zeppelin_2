pub mod gizmo_traits;
pub mod hex;
pub mod scale;
pub mod types;

pub fn plugin(app: &mut bevy::app::App) {
    app.add_plugins(scale::plugin);
}
