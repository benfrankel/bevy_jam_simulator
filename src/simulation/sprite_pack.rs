use std::ops::Range;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
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
            .init_resource::<AtlasList>()
            .add_systems(Startup, load_atlas_list)
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

    #[asset(path = "image/entity/rpg/weapons.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 9, columns = 8))]
    pub rpg_weapons: Handle<TextureAtlas>,
    #[asset(path = "image/entity/rpg/armours.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 19, columns = 9))]
    pub rpg_armours: Handle<TextureAtlas>,
}

fn random_color(mut rng: impl Rng) -> Color {
    Color::rgb(
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
    )
}

/// An atlas with metadata for generating skins
pub struct Atlas {
    path: &'static str,
    // TODO: Include Option<Color> per tile (None => any color chosen randomly)
    tiles: Vec<usize>,
}

impl Atlas {
    fn random_with_color(&self, color: Color, mut rng: impl Rng) -> Skin {
        Skin {
            atlas_path: self.path,
            index: *self.tiles.choose(&mut rng).unwrap(),
            color,
        }
    }

    fn random(&self, mut rng: impl Rng) -> Skin {
        self.random_with_color(random_color(&mut rng), rng)
    }

    fn random_white(&self, rng: impl Rng) -> Skin {
        self.random_with_color(Color::WHITE, rng)
    }
}

/// A list of all available atlas metadata
#[derive(Resource, Default)]
pub struct AtlasList(Vec<Atlas>);

fn load_atlas_list(mut atlas_list: ResMut<AtlasList>) {
    atlas_list.0.extend([
        Atlas {
            path: "none",
            tiles: vec![0],
        },
        // 1-bit
        Atlas {
            path: "one_bit_food",
            tiles: (0..5 * 17).collect(),
        },
        Atlas {
            path: "one_bit_weapons",
            tiles: (0..11 * 13).collect(),
        },
        // RPG
        Atlas {
            path: "rpg_weapons",
            tiles: (0..9 * 8).collect(),
        },
        Atlas {
            path: "rpg_armours",
            tiles: (0..19 * 9).collect(),
        },
    ]);
}

/// A thematically-consistent skin-generating space
#[derive(Default, Copy, Clone)]
#[allow(dead_code)]
pub enum SpritePack {
    #[default]
    None,
    OneBit,
    Rpg,
}

impl SpritePack {
    fn atlases(&self) -> Range<usize> {
        match self {
            Self::None => 0..1,
            Self::OneBit => 1..3,
            Self::Rpg => 3..5,
        }
    }
}

/// A single entity skin
#[derive(Default, PartialEq, Clone, Copy)]
pub struct Skin {
    atlas_path: &'static str,
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
            assets
                .field(self.atlas_path)
                .unwrap()
                .as_any()
                .downcast_ref::<Handle<TextureAtlas>>()
                .unwrap()
                .clone(),
        )
    }
}

/// A set of entity skins
pub struct SkinSet {
    pub asset_pack: SpritePack,
    pub skins: Vec<Skin>,
}

impl Default for SkinSet {
    fn default() -> Self {
        Self {
            asset_pack: default(),
            skins: vec![
                Skin {
                    atlas_path: "none",
                    index: 0,
                    color: Color::WHITE,
                },
                Skin {
                    atlas_path: "none",
                    index: 0,
                    color: Color::BLACK,
                },
            ],
        }
    }
}

impl SkinSet {
    pub fn new(
        atlas_list: &AtlasList,
        atlas_pack: SpritePack,
        count: usize,
        mut rng: impl Rng,
    ) -> Self {
        let mut this = Self {
            asset_pack: atlas_pack,
            skins: Vec::with_capacity(count),
        };
        for _ in 0..count {
            this.add_skin(atlas_list, &mut rng);
        }
        this
    }

    pub fn add_skin(&mut self, atlas_list: &AtlasList, mut rng: impl Rng) {
        let atlases = self
            .asset_pack
            .atlases()
            .map(|idx| &atlas_list.0[idx])
            .collect::<Vec<_>>();
        let atlas = atlases.choose(&mut rng).unwrap();

        // TODO: Prevent duplicates
        match self.asset_pack {
            SpritePack::None => {
                self.skins.push(atlas.random(&mut rng));
            },
            SpritePack::OneBit => {
                self.skins.push(atlas.random_white(&mut rng));
            },
            SpritePack::Rpg => {
                self.skins.push(atlas.random_white(&mut rng));
            },
        };
    }

    pub fn bundle(
        &self,
        assets: &SpritePackAssets,
        size: Vec2,
        mut rng: impl Rng,
    ) -> (TextureAtlasSprite, Handle<TextureAtlas>) {
        self.skins
            .choose(&mut rng)
            .map(|skin| skin.bundle(assets, size))
            .unwrap_or_default()
    }

    pub fn replace_sprite_pack(
        &mut self,
        atlas_list: &AtlasList,
        atlas_pack: SpritePack,
        rng: impl Rng,
    ) {
        *self = SkinSet::new(atlas_list, atlas_pack, self.skins.len(), rng);
    }
}

/// Sent when the skin set has changed and should be re-applied
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

        commands
            .entity(entity)
            .insert(simulation.skin_set.bundle(&assets, size, &mut rng));
    }
}
