use bevy::prelude::*;

use crate::board_data::BoardData;
use crate::colors;
use crate::{BoardSystems, GameState};

// TODO: ADD UI for turn tracking
// TODO: Multiplayer: Add a very low timer for turn

#[derive(Resource, Deref, DerefMut)]
pub struct FontSpaceGrotesk(pub Handle<Font>);

pub enum FontAsset {
    SpaceGrotesk,
}
impl AsRef<str> for FontAsset {
    fn as_ref(&self) -> &str {
        match self {
            FontAsset::SpaceGrotesk => "fonts/SpaceGrotesk-Medium.ttf",
        }
    }
}

#[derive(Component)]
pub struct HomeScreen;

#[derive(Component, Clone, Copy)]
pub struct StartGameButton {
    color: Color,
    border_color: Color,
    text_color: Color,
}

pub fn home_screen(mut cmds: Commands, font: Res<FontSpaceGrotesk>) {
    let start_button = StartGameButton {
        color: colors::GREEN_YELLOW.into(),
        border_color: colors::DODGER_BLUE.into(),
        text_color: colors::DEEP_PINK.into(),
    };

    cmds.spawn((
        HomeScreen,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceAround,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ZIndex(1),
    ))
    .with_children(|parent| {
        parent.spawn((
            Label,
            Text::new("Tic Tac Toe"),
            TextFont {
                font: font.clone(),
                font_size: 25.0,
                ..default()
            },
            TextColor(colors::GOLD.into()),
            BoxShadow {
                color: colors::DARK_VIOLET.into(),
                x_offset: Val::Percent(1.),
                y_offset: Val::Percent(2.5),
                spread_radius: Val::Percent(1.),
                blur_radius: Val::Px(7.5),
            },
        ));
        parent
            .spawn((
                Button,
                start_button,
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Px(30.0),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    bottom: Val::Percent(25.),
                    ..default()
                },
                BorderRadius::all(Val::Px(10.0)),
                BorderColor(start_button.border_color),
                BackgroundColor(start_button.color),
            ))
            .with_child((
                Text::new("Start"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(start_button.text_color),
            ));
    });
}

pub fn home_screen_cleanup(mut cmds: Commands, home_screen: Query<Entity, With<HomeScreen>>) {
    if let Ok(home_screen) = home_screen.get_single() {
        cmds.entity(home_screen).despawn_recursive();
    }
}

#[allow(clippy::type_complexity)]
pub fn home_screen_interaction(
    mut interaction_query: Query<
        (
            &StartGameButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (start_button, interaction, mut color, mut border_color, children) in &mut interaction_query
    {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::WHITE.into();
                border_color.0 = start_button.text_color;
                game_state.set(GameState::Start);
            }
            Interaction::Hovered => {
                *color = start_button.text_color.into();
                *text_color = start_button.color.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = start_button.color.into();
                *text_color = start_button.text_color.into();
                border_color.0 = start_button.border_color;
            }
        }
    }
}

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component, Clone, Copy)]
pub struct GoBackButton {
    color: Color,
    border_color: Color,
    text_color: Color,
}

pub fn game_over_screen(mut cmds: Commands, font: Res<FontSpaceGrotesk>, board: Res<BoardData>) {
    let leave_button = GoBackButton {
        color: colors::GOLD.into(),
        border_color: colors::DEEP_PINK.into(),
        text_color: colors::DARK_VIOLET.into(),
    };

    cmds.spawn((
        GameOverScreen,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceAround,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.975)),
        ZIndex(2),
    ))
    .with_children(|parent| {
        parent.spawn((
            Label,
            Text::new("Match Over"),
            TextFont {
                font: font.clone(),
                font_size: 25.0,
                ..default()
            },
            TextColor(colors::GOLD.into()),
        ));
        parent.spawn((
            Label,
            Text::new(board.result().unwrap().as_ref()),
            TextFont {
                font: font.clone(),
                font_size: 25.0,
                ..default()
            },
            TextColor(colors::GREEN_YELLOW.into()),
            BoxShadow {
                color: colors::DARK_VIOLET.into(),
                spread_radius: Val::Percent(60.),
                blur_radius: Val::Px(30.0),
                ..default()
            },
        ));
        parent
            .spawn((
                Button,
                leave_button,
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Px(35.0),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(10.0)),
                BorderColor(leave_button.border_color),
                BackgroundColor(leave_button.color),
            ))
            .with_child((
                Text::new("Go Back"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(leave_button.text_color),
            ));
    });
}

pub fn game_over_screen_cleanup(
    mut cmds: Commands,
    game_over_screen: Query<Entity, With<GameOverScreen>>,
    board_systems: Res<BoardSystems>,
) {
    if let Ok(game_over_screen) = game_over_screen.get_single() {
        cmds.entity(game_over_screen).despawn_recursive();
    }

    cmds.run_system(board_systems["clear_board"]);
}

#[allow(clippy::type_complexity)]
pub fn game_over_screen_interaction(
    mut interaction_query: Query<
        (
            &GoBackButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (leave_button, interaction, mut color, mut border_color, children) in &mut interaction_query
    {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::WHITE.into();
                *border_color = leave_button.text_color.into();
                game_state.set(GameState::Idle);
            }
            Interaction::Hovered => {
                *color = leave_button.text_color.into();
                *text_color = leave_button.color.into();
                *border_color = Color::WHITE.into();
            }
            Interaction::None => {
                *color = leave_button.color.into();
                *text_color = leave_button.text_color.into();
                *border_color = leave_button.border_color.into();
            }
        }
    }
}
