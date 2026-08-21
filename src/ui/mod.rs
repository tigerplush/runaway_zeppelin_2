use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, states::AppStates};

#[derive(Component)]
pub struct UiRoot;

#[derive(Asset, Clone, Reflect, Resource)]
pub struct FontHandles {
    header: Handle<Font>,
    text: Handle<Font>,
}

impl FromWorld for FontHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            header: asset_server.load("ui/MunchausenNF.ttf"),
            text: asset_server.load("ui/estre.ttf"),
        }
    }
}

#[derive(Asset, Clone, Reflect, Resource)]
pub struct ButtonHandles {
    button_background: Handle<Image>,
    slicer: TextureSlicer,
}

impl ButtonHandles {
    fn primary(&self) -> ImageNode {
        ImageNode {
            image: self.button_background.clone(),
            image_mode: NodeImageMode::Sliced(self.slicer.clone()),
            ..default()
        }
    }
}

const BUTTON_SLICE_HORIZONTAL: f32 = 70.0;
const BUTTON_SLICE_VERTICAL: f32 = 8.0;

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

#[derive(Asset, Clone, Reflect, Resource)]
struct StatusBarHandles {
    background_image: Handle<Image>,
}

impl FromWorld for StatusBarHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            background_image: asset_server.load("ui/graphics/header_frame.png"),
        }
    }
}

#[derive(Component, PartialEq)]
enum UiSlot {
    StatusBar,
    Action,
}

fn setup(status_bar_handles: Res<StatusBarHandles>, mut commands: Commands) {
    commands.spawn((
        UiRoot,
        Name::from("UiRoot"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        children![
            (
                UiSlot::StatusBar,
                Name::from("StatusBar"),
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
                ImageNode {
                    image: status_bar_handles.background_image.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                }
            ),
            (
                UiSlot::Action,
                Name::from("ActionBar"),
                Node { ..default() }
            )
        ],
    ));
}

#[derive(Component)]
pub enum AttachToUiSlot {
    StatusBar,
    Action,
}

#[derive(Component)]
struct NeedsPlacement;

#[derive(Component)]
enum FontChoice {
    Header,
    Content,
}

#[derive(Component)]
struct NeedsStyling;

pub fn primary_button(
    header: impl Into<String>,
    content: Option<String>,
    slot: AttachToUiSlot,
) -> impl Bundle {
    let header = header.into();
    (
        Name::from(format!("{} Button", header)),
        Button,
        NeedsStyling,
        NeedsPlacement,
        Node { ..default() },
        slot,
        children![(
            Node {
                padding: UiRect::axes(
                    Val::Px(BUTTON_SLICE_HORIZONTAL),
                    Val::Px(BUTTON_SLICE_VERTICAL)
                ),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Children::spawn((
                Spawn((Name::from("Header"), Text(header), FontChoice::Header)),
                SpawnIter(content.into_iter().map(|content| (
                    Name::from("Content"),
                    Text(content),
                    FontChoice::Content
                )),),
            )),
        )],
    )
}

pub fn label(label: impl Into<String>, slot: AttachToUiSlot) -> impl Bundle {
    let label = label.into();
    (
        Name::from(format!("{} Label", label)),
        Text(label),
        NeedsPlacement,
        slot,
    )
}

fn on_add_needs_styling(
    trigger: On<Add, NeedsStyling>,
    button_handles: Res<ButtonHandles>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .insert(button_handles.primary())
        .remove::<NeedsStyling>();
}

fn on_add_needs_placement(
    trigger: On<Add, NeedsPlacement>,
    query: Query<&AttachToUiSlot>,
    ui_slots: Query<(Entity, &UiSlot)>,
    mut commands: Commands,
) {
    let Ok(attach_to) = query.get(trigger.entity) else {
        return;
    };

    let parent_maybe = match attach_to {
        AttachToUiSlot::Action => ui_slots.iter().find(|&(_, slot)| &UiSlot::Action == slot),
        AttachToUiSlot::StatusBar => ui_slots
            .iter()
            .find(|&(_, slot)| &UiSlot::StatusBar == slot),
    };

    let Some((parent, _)) = parent_maybe else {
        return;
    };

    commands
        .entity(trigger.entity)
        .insert(ChildOf(parent))
        .remove::<NeedsPlacement>();
}

fn on_add_text(
    trigger: On<Add, Text>,
    font_handles: Res<FontHandles>,
    query: Query<&FontChoice>,
    mut commands: Commands,
) {
    if let Ok(font_choice) = query.get(trigger.entity) {
        let text_font = match font_choice {
            FontChoice::Header => TextFont {
                font: FontSource::Handle(font_handles.header.clone()),
                ..default()
            },
            FontChoice::Content => TextFont {
                font: FontSource::Handle(font_handles.text.clone()),
                ..default()
            },
        };
        commands.entity(trigger.entity).insert(text_font);
    }
}

pub fn plugin(app: &mut App) {
    app.load_resource::<FontHandles>()
        .load_resource::<ButtonHandles>()
        .load_resource::<StatusBarHandles>()
        .add_systems(OnEnter(AppStates::InGame), setup)
        .add_observer(on_add_needs_styling)
        .add_observer(on_add_needs_placement)
        .add_observer(on_add_text);
}
