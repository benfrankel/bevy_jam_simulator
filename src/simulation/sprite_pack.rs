use std::ops::Range;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

use crate::simulation::Simulation;
use crate::util::gen_color;
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

    #[asset(path = "image/entity/rpg/armours.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 19, columns = 9))]
    pub rpg_armours: Handle<TextureAtlas>,
    #[asset(path = "image/entity/rpg/books.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 12, columns = 16))]
    pub rpg_books: Handle<TextureAtlas>,
    #[asset(path = "image/entity/rpg/chests.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 6, columns = 8))]
    pub rpg_chests: Handle<TextureAtlas>,
    #[asset(path = "image/entity/rpg/consumables.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 17, columns = 44))]
    pub rpg_consumables: Handle<TextureAtlas>,
    #[asset(path = "image/entity/rpg/potions.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 15, columns = 21))]
    pub rpg_potions: Handle<TextureAtlas>,
    #[asset(path = "image/entity/rpg/weapons.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 9, columns = 8))]
    pub rpg_weapons: Handle<TextureAtlas>,
}

/// A list of all available atlas metadata
#[derive(Resource, Default)]
pub struct AtlasList(Vec<Atlas>);

fn load_atlas_list(mut atlas_list: ResMut<AtlasList>) {
    atlas_list.0.extend([
        Atlas {
            path: "none",
            tiles: vec![Tile::any_color(0)],
        },
        // 1-bit
        // TODO: Curate tiles and colors
        Atlas {
            path: "one_bit_food",
            tiles: (0..5 * 17).map(Tile::any_color).collect(),
        },
        Atlas {
            path: "one_bit_weapons",
            tiles: (0..11 * 13).map(Tile::any_color).collect(),
        },
        // RPG
        Atlas {
            path: "rpg_armours",
            tiles: (0..19 * 9).map(Tile::new).collect(),
        },
        Atlas {
            path: "rpg_books",
            tiles: (0..12 * 16).map(Tile::new).collect(),
        },
        Atlas {
            path: "rpg_chests",
            tiles: (0..6 * 8).map(Tile::new).collect(),
        },
        Atlas {
            path: "rpg_consumables",
            tiles: (0..12 * 44 + 22)
                .chain(13 * 44..13 * 44 + 22)
                .chain(14 * 44..14 * 44 + 22)
                .chain(15 * 44..15 * 44 + 22)
                .chain(16 * 44..16 * 44 + 22)
                .map(Tile::new)
                .collect(),
        },
        Atlas {
            path: "rpg_potions",
            tiles: (0..15 * 21).map(Tile::new).collect(),
        },
        Atlas {
            path: "rpg_weapons",
            tiles: (0..9 * 8).map(Tile::new).collect(),
        },
    ]);
}

/// An atlas tile that can be used for generating skins
struct Tile {
    index: usize,
    /// The color of the generated skin, or a random color if None
    color: Option<Color>,
}

impl Tile {
    fn new(index: usize) -> Self {
        Self {
            index,
            color: Some(Color::WHITE),
        }
    }

    fn any_color(index: usize) -> Self {
        Self { index, color: None }
    }
}

/// An atlas with metadata for generating skins
pub struct Atlas {
    path: &'static str,
    tiles: Vec<Tile>,
}

impl Atlas {
    fn random(&self, mut rng: impl Rng) -> Skin {
        let tile = self.tiles.choose(&mut rng).unwrap();
        Skin {
            atlas_path: self.path,
            index: tile.index,
            color: tile.color.unwrap_or_else(|| gen_color(&mut rng)),
        }
    }
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
    fn atlas_range(&self) -> Range<usize> {
        match self {
            Self::None => 0..1,
            Self::OneBit => 1..3,
            Self::Rpg => 3..9,
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
        let atlas = self
            .asset_pack
            .atlas_range()
            .map(|idx| &atlas_list.0[idx])
            .choose(&mut rng)
            .unwrap();

        // TODO: Prevent duplicates
        self.skins.push(atlas.random(&mut rng));
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
