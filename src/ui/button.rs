use bevy::prelude::*;

pub const BUTTON_SLICE_HORIZONTAL: f32 = 70.0;
pub const BUTTON_SLICE_VERTICAL: f32 = 8.0;

#[derive(Asset, Clone, Reflect, Resource)]
pub struct ButtonHandles {
    #[dependency]
    button_background: Handle<Image>,
    slicer: TextureSlicer,
}

impl ButtonHandles {
    pub fn primary(&self) -> ImageNode {
        ImageNode {
            image: self.button_background.clone(),
            image_mode: NodeImageMode::Sliced(self.slicer.clone()),
            ..default()
        }
    }
}


impl FromWorld for ButtonHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let slicer = TextureSlicer {
            border: BorderRect::axes(BUTTON_SLICE_HORIZONTAL, BUTTON_SLICE_VERTICAL),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        };
        Self {
            button_background: asset_server.load("ui/graphics/btn_middle_2.png"),
            slicer,
        }
    }
}