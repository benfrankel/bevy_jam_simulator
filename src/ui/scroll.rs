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

#[derive(Component, Reflect)]
pub struct ScrollContent {
    pub position: f32,
    pub sensitivity: f32,
    pub scrollbar: Entity,
}

fn mouse_scroll(
    mut events: EventReader<MouseWheel>,
    mut scroll_query: Query<(&mut ScrollContent, &mut Style, &Parent, &Node)>,
    mut scrollbar_query: Query<&mut Style, Without<ScrollContent>>,
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
        if parent_height <= 0.0 {
            return;
        }
        let overflow = (height - parent_height).max(0.0);
        scroll.position = scroll.position.clamp(-overflow, 0.0);

        style.top = Val::Px(scroll.position);

        // Update scrollbar
        if let Ok(mut style) = scrollbar_query.get_mut(scroll.scrollbar) {
            let height = (1.0 - overflow / parent_height).max(0.1);
            let top = (-scroll.position / parent_height).min(1.0 - height);
            style.height = Val::Percent(100.0 * height);
            style.top = Val::Percent(100.0 * top);
        }
    }
}
