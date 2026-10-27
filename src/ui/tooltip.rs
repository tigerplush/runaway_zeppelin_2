use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

#[derive(Asset, Clone, Reflect, Resource)]
pub struct PopupWindowHandles {
    #[dependency]
    popup_window_background: Handle<Image>,
    slicer: TextureSlicer,
}

impl PopupWindowHandles {
    fn image_node(&self) -> ImageNode {
        ImageNode {
            image: self.popup_window_background.clone(),
            image_mode: NodeImageMode::Sliced(self.slicer.clone()),
            ..default()
        }
    }
}

impl FromWorld for PopupWindowHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let slicer = TextureSlicer {
            border: BorderRect::all(10.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        };
        Self {
            popup_window_background: asset_server.load("ui/graphics/window_popup.png"),
            slicer,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Tooltip(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TooltipContent(Entity);

pub fn tooltip(parent: Entity, content: impl Bundle) -> impl Bundle {
    (
        Name::from(format!("Tooltip Content to {}", parent)),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        TooltipContent(parent),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            content,
        )],
    )
}

pub fn on_add_tooltip(
    trigger: On<Add<TooltipContent>>,
    popup_window: Res<PopupWindowHandles>,
    tooltip_contents: Query<&TooltipContent>,
    mut commands: Commands,
) {
    let Ok(content) = tooltip_contents.get(trigger.entity) else {
        return;
    };

    commands
        .entity(trigger.entity)
        .insert(popup_window.image_node());
    commands.entity(content.0).insert(Tooltip(trigger.entity));
}

pub fn on_enter_tooltip(
    trigger: On<PointerEnter>,
    tooltips: Query<(&ComputedNode, &Tooltip), Without<TooltipContent>>,
    mut contents: Query<(&mut Visibility, &mut Node), With<TooltipContent>>,
) {
    let Ok((computed, tooltip)) = tooltips.get(trigger.entity) else {
        return;
    };
    let Ok((mut visibility, mut node)) = contents.get_mut(tooltip.0) else {
        return;
    };

    *visibility = Visibility::Visible;

    let half_size = computed.size / 2.0;

    let top_left = half_size;
    node.left = Val::Px(top_left.x);
    node.top = Val::Px(top_left.y);
}

pub fn on_leave_tooltip(
    trigger: On<PointerLeave>,
    tooltips: Query<&Tooltip>,
    mut contents: Query<&mut Visibility, With<TooltipContent>>,
) {
    let Ok(tooltip) = tooltips.get(trigger.entity) else {
        return;
    };
    let Ok(mut content) = contents.get_mut(tooltip.0) else {
        return;
    };

    *content = Visibility::Hidden;
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Tooltip>()
        .register_type::<TooltipContent>()
        .load_resource::<PopupWindowHandles>()
        .add_observer(on_add_tooltip)
        .add_observer(on_enter_tooltip)
        .add_observer(on_leave_tooltip);
}
