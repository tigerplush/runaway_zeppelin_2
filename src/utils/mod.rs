pub mod gizmo_traits;
pub mod hex;
pub mod scale;
#[cfg(debug_assertions)]
pub mod shader;
pub mod types;

pub trait SmoothStep {
    fn smoothstep(&self, edge0: Self, edge1: Self) -> Self;
}

impl SmoothStep for f32 {
    fn smoothstep(&self, edge0: Self, edge1: Self) -> Self {
        let x = (self - edge0) / (edge1 - edge0);
        let x = x.clamp(0_f32, 1_f32);
        x * x * (3_f32 - 2_f32 * x)
    }
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_plugins(scale::plugin);

    #[cfg(debug_assertions)]
    app.add_plugins(shader::plugin);
}
