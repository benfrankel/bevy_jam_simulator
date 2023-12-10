use std::ops::Range;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
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
    // None
    #[asset(texture_atlas(tile_size_x = 1.0, tile_size_y = 1.0, rows = 1, columns = 1))]
    #[asset(path = "image/entity/none.png")]
    pub none: Handle<TextureAtlas>,

    // Text
    #[asset(texture_atlas(tile_size_x = 12.0, tile_size_y = 12.0, rows = 1, columns = 94))]
    #[asset(path = "image/entity/text/PyriousPixel-B.png")]
    pub text: Handle<TextureAtlas>,

    // 1-bit
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 17, columns = 11))]
    #[asset(path = "image/entity/1-bit/Clothing.png")]
    pub one_bit_clothing: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 18, columns = 14))]
    #[asset(path = "image/entity/1-bit/Creatures.png")]
    pub one_bit_creatures: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 5, columns = 17))]
    #[asset(path = "image/entity/1-bit/Food.png")]
    pub one_bit_food: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 5, columns = 8))]
    #[asset(path = "image/entity/1-bit/Gems-Jewels-and-Money.png")]
    pub one_bit_gems_jewels_and_money: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 4, columns = 8))]
    #[asset(path = "image/entity/1-bit/Instruments.png")]
    pub one_bit_instruments: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 9, columns = 17))]
    #[asset(path = "image/entity/1-bit/Jewelry.png")]
    pub one_bit_jewelry: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 9, columns = 4))]
    #[asset(path = "image/entity/1-bit/Misc-Future.png")]
    pub one_bit_misc_future: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 10, columns = 6))]
    #[asset(path = "image/entity/1-bit/Misc.png")]
    pub one_bit_misc: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 27, columns = 32))]
    #[asset(path = "image/entity/1-bit/People.png")]
    pub one_bit_people: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 6, columns = 4))]
    #[asset(path = "image/entity/1-bit/Potions.png")]
    pub one_bit_potions: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 8, columns = 9))]
    #[asset(path = "image/entity/1-bit/Tools.png")]
    pub one_bit_tools: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 19, columns = 21))]
    #[asset(path = "image/entity/1-bit/UI.png")]
    pub one_bit_ui: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 11, columns = 13))]
    #[asset(path = "image/entity/1-bit/Weapons.png")]
    pub one_bit_weapons: Handle<TextureAtlas>,

    // RPG
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

    // Ninja
    #[asset(path = "image/entity/ninja/Animals.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 1, columns = 7))]
    pub ninja_animals: Handle<TextureAtlas>,
    #[asset(path = "image/entity/ninja/Characters.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 1, columns = 56))]
    pub ninja_characters: Handle<TextureAtlas>,
    #[asset(path = "image/entity/ninja/Items.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 1, columns = 2))]
    pub ninja_items: Handle<TextureAtlas>,
    #[asset(path = "image/entity/ninja/Monsters.png")]
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, rows = 1, columns = 13))]
    pub ninja_monsters: Handle<TextureAtlas>,
}

/// A list of all available atlas metadata
#[derive(Resource, Default)]
pub struct AtlasList(Vec<Atlas>);

fn load_atlas_list(mut atlas_list: ResMut<AtlasList>) {
    atlas_list.0.extend([
        // None
        Atlas {
            path: "none",
            tiles: vec![Tile::any_color(0)],
        },
        // Text
        Atlas {
            path: "text",
            tiles: (0..94).map(Tile::any_color).collect(),
        },
        // 1-bit
        Atlas {
            path: "one_bit_clothing",
            tiles: [14, 19, 31].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_creatures",
            tiles: [
                10, 12, 15, 17, 18, 27, 30, 46, 55, 58, 75, 93, 96, 107, 128, 140, 166,
            ]
            .map(Tile::new)
            .into(),
        },
        Atlas {
            path: "one_bit_food",
            tiles: [5, 12, 19, 22, 28, 35, 38, 39, 43, 55, 68, 69, 72]
                .map(Tile::new)
                .into(),
        },
        Atlas {
            path: "one_bit_gems_jewels_and_money",
            tiles: [0, 8].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_instruments",
            tiles: [0].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_jewelry",
            tiles: [24, 88].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_misc_future",
            tiles: [32].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_misc",
            tiles: [42, 49, 55].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_people",
            tiles: [454, 643].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_potions",
            tiles: [10].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_tools",
            tiles: [0, 1, 2, 4, 10, 13, 16, 30, 34, 63].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_ui",
            tiles: [157, 177, 231, 315].map(Tile::new).into(),
        },
        Atlas {
            path: "one_bit_weapons",
            tiles: [7, 28, 44, 92, 122, 140].map(Tile::new).into(),
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
        // Ninja
        Atlas {
            path: "ninja_animals",
            tiles: (0..7).map(Tile::new).collect(),
        },
        Atlas {
            path: "ninja_characters",
            tiles: (0..56).map(Tile::new).collect(),
        },
        Atlas {
            path: "ninja_items",
            tiles: (0..2).map(Tile::new).collect(),
        },
        Atlas {
            path: "ninja_monsters",
            tiles: (0..13).map(Tile::new).collect(),
        },
    ]);
}

/// A thematically-consistent skin-generating space
#[derive(Default, Copy, Clone)]
#[allow(dead_code)]
pub enum SpritePack {
    #[default]
    None,
    Text,
    OneBit,
    Rpg,
    Ninja,
}

impl SpritePack {
    fn atlases(&self) -> Range<usize> {
        match self {
            Self::None => 0..1,
            Self::Text => 1..2,
            Self::OneBit => 2..15,
            Self::Rpg => 15..21,
            Self::Ninja => 21..25,
        }
    }

    fn random(&self, atlas_list: &AtlasList, mut rng: impl Rng) -> Skin {
        let atlas_idx = *self
            .atlases()
            .collect::<Vec<_>>()
            .choose_weighted(&mut rng, |&i| atlas_list.0[i].tiles.len())
            .unwrap();

        atlas_list.0[atlas_idx].random(&mut rng)
    }
}

/// A view into an atlas that can be used for generating skins
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
    pub sprite_pack: SpritePack,
    pub skins: Vec<Skin>,
}

impl Default for SkinSet {
    fn default() -> Self {
        Self {
            sprite_pack: default(),
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
        sprite_pack: SpritePack,
        count: usize,
        mut rng: impl Rng,
    ) -> Self {
        let mut this = Self {
            sprite_pack,
            skins: Vec::with_capacity(count),
        };
        for _ in 0..count {
            this.add_skin(atlas_list, &mut rng);
        }
        this
    }

    /// Returns false if it fails to add a non-duplicate skin
    pub fn add_skin(&mut self, atlas_list: &AtlasList, mut rng: impl Rng) -> bool {
        // Make a reasonable attempt to prevent duplicates
        const MAX_ATTEMPTS: usize = 64;
        for _ in 0..MAX_ATTEMPTS {
            let skin = self.sprite_pack.random(atlas_list, &mut rng);
            if !self.skins.contains(&skin) {
                self.skins.push(skin);
                return true;
            }
        }

        false
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
        sprite_pack: SpritePack,
        rng: impl Rng,
    ) {
        *self = SkinSet::new(atlas_list, sprite_pack, self.skins.len(), rng);
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
