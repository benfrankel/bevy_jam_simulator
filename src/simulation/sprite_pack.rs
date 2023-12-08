use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct SpritePackPlugin;

impl Plugin for SpritePackPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpritePackAssets>()
            .init_collection::<SpritePackAssets>();
    }
}

pub enum SpritePack {
    None(Vec<Color>),
    OneBit(Vec<(Color, usize)>),
}

impl Default for SpritePack {
    fn default() -> Self {
        Self::None(vec![Color::BLACK, Color::WHITE])
    }
}

impl SpritePack {
    pub fn add_skin(&mut self, mut rng: impl Rng) {
        let color = Color::Rgba {
            red: rng.gen_range(0.0..1.0),
            green: rng.gen_range(0.0..1.0),
            blue: rng.gen_range(0.0..1.0),
            alpha: 1.0,
        };
        match self {
            Self::None(colors) => {
                colors.push(color);
            },
            // TODO: Curate the tiles.
            // FIXME: Prevent duplicates.
            Self::OneBit(tiles) => tiles.push((color, rng.gen_range(0..=5))),
        }
    }

    pub fn apply(
        &self,
        commands: &mut Commands,
        entity: Entity,
        assets: &SpritePackAssets,
        size: Vec2,
        mut rng: impl Rng,
    ) {
        match self {
            Self::None(colors) => {
                commands.entity(entity).insert((
                    Sprite {
                        color: *colors.choose(&mut rng).unwrap(),
                        custom_size: Some(size),
                        ..default()
                    },
                    SpriteBundle::default().texture,
                ));
            },
            Self::OneBit(tiles) => {
                let (color, index) = *tiles.choose(&mut rng).unwrap();
                commands.entity(entity).insert((
                    TextureAtlasSprite {
                        color,
                        index,
                        custom_size: Some(size),
                        ..default()
                    },
                    assets.one_bit_food.clone(),
                ));
            },
        };
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SpritePackAssets {
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 5, columns = 17))]
    #[asset(path = "image/entity/1-bit/Food.png")]
    pub one_bit_food: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 10.0, tile_size_y = 10.0, rows = 11, columns = 13))]
    #[asset(path = "image/entity/1-bit/Weapons.png")]
    pub one_bit_weapons: Handle<TextureAtlas>,
}
