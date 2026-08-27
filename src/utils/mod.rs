pub mod gizmo_traits;
pub mod hex;
pub mod scale;
#[cfg(debug_assertions)]
pub mod shader;
pub mod types;

pub fn plugin(app: &mut bevy::app::App) {
    app.add_plugins(scale::plugin);

    #[cfg(debug_assertions)]
    app.add_plugins(shader::plugin);
}
