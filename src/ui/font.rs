use bevy::prelude::*;

#[derive(Component)]
pub enum FontChoice {
    Header,
    Content,
}

#[derive(Asset, Clone, Reflect, Resource)]
pub(super) struct FontHandles {
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

pub(super) fn on_add_text(
    trigger: On<Add<Text>>,
    font_handles: Res<FontHandles>,
    query: Query<&FontChoice, Without<TextFont>>,
    mut already_existing: Query<(&FontChoice, &mut TextFont), With<TextFont>>,
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

    if let Ok((font_choice, mut font)) = already_existing.get_mut(trigger.entity) {
        font.font = FontSource::Handle(match font_choice {
            FontChoice::Header => font_handles.header.clone(),
            FontChoice::Content => font_handles.text.clone(),
        });
    }
}
