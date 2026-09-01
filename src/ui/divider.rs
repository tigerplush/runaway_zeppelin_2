use bevy::prelude::*;

#[derive(Asset, Clone, Reflect, Resource)]
pub struct DividerHandles {
    #[dependency]
    full: Handle<Image>,
    #[dependency]
    half: Handle<Image>,
}

impl DividerHandles {
    pub fn full(&self) -> ImageNode {
        ImageNode {
            image: self.full.clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        }
    }

    pub fn half(&self, flip_x: bool) -> ImageNode {
        ImageNode {
            image: self.half.clone(),
            flip_x,
            ..default()
        }
    }
}


impl FromWorld for DividerHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            full: asset_server.load("ui/graphics/sep_goldbar.png"),
            half: asset_server.load("ui/graphics/sep_open.png"),
        }
    }
}