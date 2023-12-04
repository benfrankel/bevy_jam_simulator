use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde::Deserialize;
use serde::Serialize;

use super::FontSize;
use super::FONT_HANDLE;
use crate::config::Config;
use crate::AppRoot;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Tooltip>()
            .add_systems(Startup, spawn_tooltip)
            .add_systems(Update, show_tooltip_on_hover);
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct TooltipConfig {
    max_width: Val,
    background_color: Color,
    text_color: Color,
    font_size: Val,
}

fn spawn_tooltip(mut commands: Commands, mut root: ResMut<AppRoot>, config: Res<Config>) {
    let config = &config.tooltip;

    root.tooltip = commands
        .spawn((
            Name::new("Tooltip"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    max_width: config.max_width,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                background_color: config.background_color.into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(1000),
                ..default()
            },
        ))
        .id();

    root.tooltip_text = commands
        .spawn((
            Name::new("TooltipText"),
            TextBundle::from_section(
                "",
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.text_color,
                    ..default()
                },
            ),
            FontSize::new(config.font_size),
        ))
        .set_parent(root.tooltip)
        .id();
}

#[derive(Reflect)]
pub enum TooltipSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Component, Reflect)]
pub struct Tooltip {
    pub text: String,
    pub side: TooltipSide,
}

fn show_tooltip_on_hover(
    root: Res<AppRoot>,
    ui_scale: Res<UiScale>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut tooltip_query: Query<(&mut Visibility, &mut Style)>,
    mut tooltip_text_query: Query<&mut Text>,
    interaction_query: Query<(&Interaction, &Tooltip, &GlobalTransform, &Node)>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let Ok((mut tooltip_visibility, mut tooltip_style)) = tooltip_query.get_mut(root.tooltip)
    else {
        return;
    };
    let Ok(mut tooltip_text) = tooltip_text_query.get_mut(root.tooltip_text) else {
        return;
    };

    for (interaction, tooltip, gt, node) in &interaction_query {
        if !matches!(interaction, Interaction::Hovered) {
            *tooltip_visibility = Visibility::Hidden;
            continue;
        }

        let scale_factor = window.scale_factor();
        let rect = node.physical_rect(gt, scale_factor, ui_scale.0);

        let width = window.physical_width() as f32;
        let height = window.physical_height() as f32;
        let (left, right, top, bottom) = (rect.min.x, rect.max.x, rect.min.y, rect.max.y);

        *tooltip_visibility = Visibility::Inherited;
        tooltip_text.sections[0].value = tooltip.text.clone();
        (
            tooltip_style.left,
            tooltip_style.right,
            tooltip_style.top,
            tooltip_style.bottom,
        ) = match tooltip.side {
            TooltipSide::Left => (
                Val::Auto,
                Val::Px(width - left),
                Val::Auto,
                Val::Px(height - bottom),
            ),
            TooltipSide::Right => (
                Val::Px(right),
                Val::Auto,
                Val::Auto,
                Val::Px(height - bottom),
            ),
            TooltipSide::Top => (Val::Px(left), Val::Auto, Val::Auto, Val::Px(height - top)),
            TooltipSide::Bottom => (Val::Px(left), Val::Auto, Val::Px(bottom), Val::Auto),
        };
        return;
    }
}
