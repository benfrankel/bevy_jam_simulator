use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

use crate::simulation::Simulation;
use crate::AppRoot;
use crate::AppSet;

pub struct SpritePackPlugin;

impl Plugin for SpritePackPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpritePackAssets>()
            .register_type::<SpritePackEvent>()
            .add_event::<SpritePackEvent>()
            .init_collection::<SpritePackAssets>()
            .add_systems(
                Update,
                apply_sprite_pack
                    .in_set(AppSet::End)
                    .run_if(on_event::<SpritePackEvent>()),
            );
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SpritePackAssets {
    #[asset(texture_atlas(tile_size_x = 1.0, tile_size_y = 1.0, rows = 1, columns = 1))]
    #[asset(path = "image/entity/none.png")]
    pub none: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 5, columns = 17))]
    #[asset(path = "image/entity/1-bit/Food.png")]
    pub one_bit_food: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 11, columns = 13))]
    #[asset(path = "image/entity/1-bit/Weapons.png")]
    pub one_bit_weapons: Handle<TextureAtlas>,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum Atlas {
    #[default]
    None,
    OneBitFood,
    OneBitWeapons,
}

impl Atlas {
    fn handle(&self, assets: &SpritePackAssets) -> Handle<TextureAtlas> {
        match self {
            Self::None => &assets.none,
            Self::OneBitFood => &assets.one_bit_food,
            Self::OneBitWeapons => &assets.one_bit_weapons,
        }
        .clone()
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub struct Skin {
    atlas: Atlas,
    index: usize,
    color: Color,
}

impl Skin {
    fn bundle(
        &self,
        assets: &SpritePackAssets,
        size: Vec2,
    ) -> (TextureAtlasSprite, Handle<TextureAtlas>) {
        (
            TextureAtlasSprite {
                color: self.color,
                index: self.index,
                custom_size: Some(size),
                ..default()
            },
            self.atlas.handle(assets),
        )
    }
}

#[derive(Default, Copy, Clone)]
pub enum SkinSet {
    #[default]
    None,
    OneBit,
}

pub struct SpritePack {
    pub skin_set: SkinSet,
    pub skins: Vec<Skin>,
}

impl Default for SpritePack {
    fn default() -> Self {
        Self {
            skin_set: default(),
            skins: DEFAULT_SKINS.to_vec(),
        }
    }
}

impl SpritePack {
    pub fn new(skin_set: SkinSet, count: usize, mut rng: impl Rng) -> Self {
        let mut this = Self {
            skin_set,
            skins: Vec::with_capacity(count),
        };
        for _ in 0..count {
            this.add_skin(&mut rng);
        }
        this
    }

    pub fn add_skin(&mut self, mut rng: impl Rng) {
        let skins = match self.skin_set {
            SkinSet::None => {
                self.skins.push(Skin {
                    color: random_color(&mut rng),
                    ..default()
                });
                return;
            },
            SkinSet::OneBit => &ONE_BIT_SKIN_SET,
        };

        if let Some(&skin) = skins
            .iter()
            .filter(|skin| !self.skins.contains(skin))
            .choose(&mut rng)
        {
            self.skins.push(skin);
        };
    }

    pub fn apply(
        &self,
        commands: &mut Commands,
        entity: Entity,
        assets: &SpritePackAssets,
        size: Vec2,
        mut rng: impl Rng,
    ) {
        if let Some(&skin) = self.skins.choose(&mut rng) {
            commands.entity(entity).insert(skin.bundle(assets, size));
        };
    }

    pub fn replace_skin_set(&mut self, skin_set: SkinSet, rng: impl Rng) {
        *self = SpritePack::new(skin_set, self.skins.len(), rng);
    }
}

fn random_color(mut rng: impl Rng) -> Color {
    Color::rgb(
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
    )
}

/// Sent when the sprite pack changes and should be re-applied
#[derive(Event, Reflect)]
pub struct SpritePackEvent;

fn apply_sprite_pack(
    mut commands: Commands,
    root: Res<AppRoot>,
    simulation: Res<Simulation>,
    assets: Res<SpritePackAssets>,
    children_query: Query<&Children>,
    sprite_query: Query<&TextureAtlasSprite>,
) {
    let mut rng = thread_rng();

    for &entity in children_query.get(root.world).ok().into_iter().flatten() {
        let size = sprite_query
            .get(entity)
            .ok()
            .and_then(|sprite| sprite.custom_size)
            .unwrap_or_else(|| {
                Vec2::splat(rng.gen_range(simulation.entity_size_min..=simulation.entity_size_max))
            });

        simulation
            .sprite_pack
            .apply(&mut commands, entity, &assets, size, &mut rng);
    }
}

const DEFAULT_SKINS: [Skin; 2] = [
    Skin {
        atlas: Atlas::None,
        index: 0,
        color: Color::WHITE,
    },
    Skin {
        atlas: Atlas::None,
        index: 0,
        color: Color::BLACK,
    },
];

pub const ONE_BIT_SKIN_SET: [Skin; 5] = [
    Skin {
        atlas: Atlas::OneBitFood,
        index: 0,
        color: Color::WHITE,
    },
    Skin {
        atlas: Atlas::OneBitFood,
        index: 1,
        color: Color::WHITE,
    },
    Skin {
        atlas: Atlas::OneBitFood,
        index: 2,
        color: Color::WHITE,
    },
    Skin {
        atlas: Atlas::OneBitFood,
        index: 3,
        color: Color::WHITE,
    },
    Skin {
        atlas: Atlas::OneBitFood,
        index: 4,
        color: Color::WHITE,
    },
];
