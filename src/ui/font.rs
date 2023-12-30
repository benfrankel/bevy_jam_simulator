use bevy::asset::load_internal_binary_asset;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::AppSet;

pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            FONT_HANDLE,
            "../../assets/font/pypx.ttf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );
        load_internal_binary_asset!(
            app,
            BOLD_FONT_HANDLE,
            "../../assets/font/pypx-B.ttf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );
        load_internal_binary_asset!(
            app,
            HEADER_FONT_HANDLE,
            "../../assets/font/pypx-T.ttf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );

        app.register_type::<FontSize>()
            .add_systems(Update, scale_font_size.in_set(AppSet::End));
    }
}

pub const FONT_HANDLE: Handle<Font> =
    Handle::weak_from_u128(303551798864246209986336759745415587961);
pub const BOLD_FONT_HANDLE: Handle<Font> =
    Handle::weak_from_u128(317423448069604009516378143395193332978);
pub const HEADER_FONT_HANDLE: Handle<Font> =
    Handle::weak_from_u128(6129592437227906058946689932047862666);

#[derive(Component, Reflect)]
pub struct FontSize {
    pub size: Val,
    cache: f32,
}

impl FontSize {
    pub fn new(size: Val) -> Self {
        Self { size, cache: -1.0 }
    }
}

pub fn scale_font_size(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut font_size_query: Query<(&mut FontSize, &Node, &mut Text)>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let viewport_size = Vec2::new(window.resolution.width(), window.resolution.height());

    const STEP: f32 = 8.0;
    for (mut font_size, node, mut text) in &mut font_size_query {
        let Ok(resolved) = font_size.size.resolve(node.size().x, viewport_size) else {
            continue;
        };
        let resolved = (resolved / STEP).floor().max(1.0) * STEP;
        if font_size.cache == resolved {
            continue;
        }
        font_size.cache = resolved;

        for section in &mut text.sections {
            section.style.font_size = resolved;
        }
    }
}
