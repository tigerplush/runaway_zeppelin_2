use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, states::AppStates};

mod tooltip;

pub use tooltip::*;

#[derive(Component)]
pub struct UiRoot;

#[derive(Asset, Clone, Reflect, Resource)]
pub struct FontHandles {
    #[dependency]
    header: Handle<Font>,
    #[dependency]
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

#[derive(Asset, Clone, Reflect, Resource)]
pub struct ButtonHandles {
    #[dependency]
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
    #[dependency]
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

#[derive(Asset, Clone, Reflect, Resource)]
struct ResourceIconHandles {
    fuel: Handle<Image>,
}

impl ResourceIconHandles {
    fn get(&self, slot: ResourceSlot) -> Handle<Image> {
        match slot {
            ResourceSlot::Fuel => self.fuel.clone(),
        }
    }
}

impl FromWorld for ResourceIconHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            fuel: asset_server.load("ui/icons/fuel.png"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ResourceSlot {
    Fuel,
}

#[derive(Component, PartialEq)]
enum UiSlot {
    InGameTimeDisplay,
    Action,
    Resource(ResourceSlot),
}

fn resource_bar() -> impl Bundle {
    (
        Name::from("ResourceBar"),
        Node { ..default() },
        children![(
            Name::from("Fuel"),
            UiSlot::Resource(ResourceSlot::Fuel),
            Node { ..default() },
        )],
    )
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
        Pickable::IGNORE,
        children![
            (
                Name::from("StatusBar"),
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ImageNode {
                    image: status_bar_handles.background_image.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                children![
                    resource_bar(),
                    (
                        Name::from("InGameTimeDisplay"),
                        UiSlot::InGameTimeDisplay,
                        Node { ..default() },
                    )
                ]
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
    InGameTimeLabel,
    Action,
    ResourceBar(ResourceSlot),
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
        FontChoice::Content,
        NeedsPlacement,
        slot,
    )
}

#[derive(Component)]
struct NeedsIcon(ResourceSlot);

/// A row with this resource's icon (looked up and attached by `ui` itself -
/// callers never need to know which handle or asset path an icon comes
/// from). Attach your own content (e.g. `Text` + a marker + `Tooltip`) as a
/// child of the returned entity - `ui` only owns the icon and row layout.
pub fn resource_label(
    label: impl Into<String>,
    label_bundle: impl Bundle,
    slot: ResourceSlot,
) -> impl Bundle {
    (
        Name::from("Resource Label"),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        },
        NeedsPlacement,
        NeedsIcon(slot),
        children![(Text::new(label), FontChoice::Content, label_bundle),],
        AttachToUiSlot::ResourceBar(slot),
    )
}

pub fn labeled_resource_row(
    label: impl Into<String>,
    initial_value: impl Into<String>,
    value_bundle: impl Bundle,
) -> impl Bundle {
    (
        Node {
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(20.0),
            ..default()
        },
        children![
            (Text::new(label), FontChoice::Content),
            (Text::new(initial_value), FontChoice::Content, value_bundle),
        ],
    )
}

fn on_add_needs_icon(
    trigger: On<Add, NeedsIcon>,
    query: Query<&NeedsIcon>,
    icons: Res<ResourceIconHandles>,
    mut commands: Commands,
) {
    let Ok(needs_icon) = query.get(trigger.entity) else {
        return;
    };

    commands
        .entity(trigger.entity)
        .with_child((
            Name::from("Icon"),
            ImageNode::new(icons.get(needs_icon.0)),
            Node {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                ..default()
            },
            // Let hover (and thus any Tooltip on the parent row) pass through
            // to the row itself instead of stopping on this child.
            Pickable::IGNORE,
        ))
        .remove::<NeedsIcon>();
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
        AttachToUiSlot::InGameTimeLabel => ui_slots
            .iter()
            .find(|&(_, slot)| &UiSlot::InGameTimeDisplay == slot),
        AttachToUiSlot::ResourceBar(resource_slot) => ui_slots
            .iter()
            .find(|&(_, slot)| &UiSlot::Resource(*resource_slot) == slot),
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
    app.add_plugins(tooltip::plugin)
        .load_resource::<FontHandles>()
        .load_resource::<ButtonHandles>()
        .load_resource::<StatusBarHandles>()
        .load_resource::<ResourceIconHandles>()
        .add_systems(OnExit(AppStates::Preloading), setup)
        .load_resource::<PopupWindowHandles>()
        .add_observer(on_add_needs_styling)
        .add_observer(on_add_needs_placement)
        .add_observer(on_add_needs_icon)
        .add_observer(on_add_text.run_if(resource_exists::<FontHandles>));
}
