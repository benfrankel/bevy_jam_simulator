use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::AppSet;

pub struct ScrollPlugin;

impl Plugin for ScrollPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScrollContent>()
            .add_systems(Update, mouse_scroll.in_set(AppSet::Input));
    }
}

#[derive(Component, Reflect, Default)]
pub struct ScrollContent {
    position: f32,
    sensitivity: f32,
}

impl ScrollContent {
    pub fn with_sensitivity(sensitivity: f32) -> Self {
        Self {
            sensitivity,
            ..default()
        }
    }
}

fn mouse_scroll(
    mut events: EventReader<MouseWheel>,
    mut scroll_query: Query<(&mut ScrollContent, &mut Style, &Parent, &Node)>,
    node_query: Query<&Node>,
) {
    let pixels = events
        .read()
        .map(|event| {
            event.y
                * match event.unit {
                    MouseScrollUnit::Line => 20.0,
                    MouseScrollUnit::Pixel => 1.0,
                }
        })
        .sum::<f32>();

    for (mut scroll, mut style, parent, node) in &mut scroll_query {
        scroll.position += pixels * scroll.sensitivity;

        let height = node.size().y;
        let parent_height = node_query.get(parent.get()).unwrap().size().y;
        let max_scroll = (height - parent_height).max(0.0);
        scroll.position = scroll.position.clamp(-max_scroll, 0.0);

        style.top = Val::Px(scroll.position);
    }
}
