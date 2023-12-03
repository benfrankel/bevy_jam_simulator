use bevy::prelude::*;
use bevy::text::BreakLineOn;
use bevy_asset_loader::prelude::*;

use crate::config::Config;
use crate::state::AppState::*;
use crate::ui::vh;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::AppRoot;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameAssets>()
            .init_collection::<GameAssets>()
            .add_systems(OnEnter(Game), enter_game)
            .add_systems(OnExit(Game), exit_game)
            .add_systems(Update, update_code_view_bar.run_if(in_state(Game)));
    }
}

const TOP_BAR_TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const TOP_BAR_TEXT_STYLE: TextStyle = TextStyle {
    font: BOLD_FONT_HANDLE,
    font_size: 0.0,
    color: TOP_BAR_TEXT_COLOR,
};
const TOP_BAR_FONT_SIZE: f32 = 8.0;
const TOP_BAR_BACKGROUND_COLOR: Color = Color::rgb(0.165, 0.18, 0.184);

const CODE_BACKGROUND_COLOR: Color = Color::rgb(0.106, 0.118, 0.122);
const CODE_TEXT_COLOR: Color = Color::rgb(0.3, 0.9, 0.0);
const CODE_TEXT_STYLE: TextStyle = TextStyle {
    font: FONT_HANDLE,
    font_size: 0.0,
    color: CODE_TEXT_COLOR,
};
const CODE_FONT_SIZE: f32 = 6.0;

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GameAssets {}

#[derive(Resource)]
struct CodeModel {
    /// Lines of Code.
    loc: f64,
}

impl Default for CodeModel {
    fn default() -> Self {
        Self { loc: 0.0 }
    }
}

#[derive(Component)]
struct LinesText;

#[derive(Component)]
struct CodeText;

fn enter_game(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    commands.insert_resource(ClearColor(config.bg_color));

    commands.insert_resource(CodeModel::default());

    let code_view = commands
        .spawn((
            Name::new("CodeView"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(35.0),
                    height: Val::Percent(100.0),
                    // padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    // align_items: AlignItems::Center,
                    // justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: CODE_BACKGROUND_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    // Top bar part of the code view.
    let code_header_container = commands
        .spawn((
            Name::new("CodeHeaderContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: vh(20.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    // border: UiRect::bottom(vh(BORDER_WIDTH)),
                    ..default()
                },
                background_color: TOP_BAR_BACKGROUND_COLOR.into(),
                // border_color: BORDER_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(code_view)
        .id();

    commands
        .spawn((
            Name::new("HeaderText"),
            TextBundle::from_section("Lines: 0", TOP_BAR_TEXT_STYLE)
                .with_text_alignment(TextAlignment::Left),
            FontSize::new(vh(TOP_BAR_FONT_SIZE)),
            LinesText,
        ))
        .set_parent(code_header_container);

    // Actual content of the code view.
    let text_area_container = commands
        .spawn((
            Name::new("TextArea"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: CODE_BACKGROUND_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(code_view)
        .id();

    let mut text = TextBundle::from_section(
        "AFJLKDFJ:AFKJDA:FJDAFLKJFLJDLFJ:LDFJSLA:FJDLF",
        CODE_TEXT_STYLE,
    )
    .with_text_alignment(TextAlignment::Left);
    text.text.linebreak_behavior = BreakLineOn::AnyCharacter;
    commands
        .spawn((
            Name::new("TextAreaText"),
            text,
            FontSize::new(vh(CODE_FONT_SIZE)),
            CodeText,
        ))
        .set_parent(text_area_container);
}

fn exit_game(root: Res<AppRoot>, mut transform_query: Query<&mut Transform>) {
    let Ok(mut transform) = transform_query.get_mut(root.camera) else {
        return;
    };
    transform.translation = Vec2::ZERO.extend(transform.translation.z);
}

fn update_code_view_bar(code_model: Res<CodeModel>, mut query: Query<&mut Text, With<LinesText>>) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("Lines: {}", code_model.loc);
}
