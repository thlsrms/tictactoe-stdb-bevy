use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use spacetimedb_sdk::{DbContext, Table};

use crate::AppState;
use crate::network::{
    CreateRoom, JoinGame, LeaveGame, LeaveRoom, LobbyRoomTableAccess, MarkCell, NetworkConnection,
};
use crate::resources::{BoardData, BoardSystems, FontSpaceGrotesk};

use super::{
    CELL_SIZE, CellMarked, GameOverScreen, Grid, GridCell, JoinGameButton, LeaveGameButton,
    LobbyRoomId, LobbyRoomScreen, TopBar, TurnOwnerLabel, TurnTimeCounter,
};
use super::{HomeScreen, LobbyPanel, NewGameButton, UiButtonStyle, colors};

// TODO: Cleanup this module

pub fn setup_systems(app: &mut App) {
    // Main menu
    app.add_systems(
        OnEnter(AppState::HomeScreen),
        (home_screen, populate_lobby_from_cache).chain(),
    );
    app.add_systems(
        Update,
        (
            new_game_button_interaction,
            join_game_button_interaction,
            update_lobby_scroll_position,
        )
            .run_if(in_state(AppState::HomeScreen)),
    );

    // Loby screen (only visible to the lobby's owner)
    app.add_systems(OnEnter(AppState::LobbyScreen), lobby_room_screen);
    app.add_systems(
        Update,
        (lobby_screen_leave_interaction,).run_if(in_state(AppState::LobbyScreen)),
    );

    // Game initialization step
    app.add_systems(OnEnter(AppState::GameSetup), initialize_grid_board);

    // Game in Progress
    app.add_systems(
        Update,
        (leave_game_button_interaction, grid_cell_interaction)
            .run_if(in_state(AppState::GameInProgress)),
    );
    app.add_systems(
        FixedUpdate,
        (turn_expiration_time_update).run_if(in_state(AppState::GameInProgress)),
    );

    // Wrap up the match
    app.add_systems(OnEnter(AppState::GameOverScreen), game_over_screen);
    app.add_systems(
        Update,
        (game_over_screen_interaction,).run_if(in_state(AppState::GameOverScreen)),
    );
}

pub fn home_screen(mut cmds: Commands, font: Res<FontSpaceGrotesk>) {
    cmds.spawn((
        StateScoped(AppState::HomeScreen),
        HomeScreen,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            display: Display::Grid,
            row_gap: Val::Px(1.),
            grid_template_rows: vec![RepeatedGridTrack::percent(10, 10.)],
            ..default()
        },
        ZIndex(1),
    ))
    .with_children(|parent| {
        parent.spawn((
            Label,
            Text::new("Tic-Tac-Toe"),
            TextFont {
                font: font.clone(),
                font_size: 25.0,
                ..default()
            },
            TextColor(colors::GOLD.into()),
            Node {
                grid_row: GridPlacement::span(2),
                ..default()
            },
            BoxShadow::new(
                colors::DODGER_BLUE.with_alpha(0.5).into(),
                Val::Px(0.),
                Val::Px(-2.),
                Val::Px(2.),
                Val::Px(20.0),
            ),
        ));

        let ui_button_style = UiButtonStyle {
            color: colors::GREEN_YELLOW.into(),
            border_color: colors::DODGER_BLUE.into(),
            text_color: colors::DARK_VIOLET.into(),
        };
        parent
            .spawn((
                NewGameButton,
                Button,
                ui_button_style,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.),
                    right: Val::Px(60.),
                    height: Val::Px(30.0),
                    min_width: Val::Px(70.),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                ZIndex(3),
                BorderRadius::all(Val::Px(5.0)),
                BorderColor(ui_button_style.border_color),
                BackgroundColor(ui_button_style.color),
                BoxShadow::new(
                    colors::DODGER_BLUE.with_alpha(0.5).into(),
                    Val::Px(0.),
                    Val::Px(-2.),
                    Val::Percent(10.),
                    Val::Px(5.0),
                ),
            ))
            .with_child((
                Text::new("Create"),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(ui_button_style.text_color),
            ));

        parent.spawn((
            LobbyPanel,
            Node {
                width: Val::Percent(96.),
                height: Val::Percent(94.),
                // grid_row: GridPlacement::start_end(4, -1),
                grid_row: GridPlacement::start_span(4, 6),
                display: Display::Grid,
                row_gap: Val::Px(2.),
                overflow: Overflow::scroll_y(),
                overflow_clip_margin: OverflowClipMargin::padding_box(),
                grid_auto_rows: vec![GridTrack::px(30.)],
                grid_auto_flow: GridAutoFlow::Row,
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                margin: UiRect {
                    top: Val::Px(25.),
                    ..default()
                },
                ..default()
            },
            ZIndex(2),
            BorderRadius::all(Val::Px(5.)),
        ));
    });
}

#[allow(clippy::type_complexity)]
pub fn new_game_button_interaction(
    mut interaction_query: Query<
        (
            &UiButtonStyle,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<NewGameButton>),
    >,
    mut text_query: Query<&mut TextColor>,
    conn: Res<NetworkConnection>,
) {
    for (start_button, interaction, mut color, mut border_color, children) in &mut interaction_query
    {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::WHITE.into();
                *border_color = start_button.text_color.into();

                conn.reducers.create_room().unwrap();
            }
            Interaction::Hovered => {
                *color = start_button.text_color.into();
                *text_color = start_button.color.into();
                *border_color = Color::WHITE.into();
            }
            Interaction::None => {
                *color = start_button.color.into();
                *text_color = start_button.text_color.into();
                *border_color = start_button.border_color.into();
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn join_game_button_interaction(
    mut interaction_query: Query<
        (
            &JoinGameButton,
            &UiButtonStyle,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor>,
    conn: Res<NetworkConnection>,
) {
    for (join_game, start_button, interaction, mut color, mut border_color, children) in
        &mut interaction_query
    {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::WHITE.into();
                *border_color = start_button.text_color.into();

                conn.reducers.join_game(join_game.0).unwrap();
            }
            Interaction::Hovered => {
                *color = start_button.text_color.into();
                *text_color = start_button.color.into();
                *border_color = Color::WHITE.into();
            }
            Interaction::None => {
                *color = start_button.color.into();
                *text_color = start_button.text_color.into();
                *border_color = start_button.border_color.into();
            }
        }
    }
}

pub fn update_lobby_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut scrolled_node_query: Query<&mut ScrollPosition, With<LobbyPanel>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let dy = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * 15.,
            MouseScrollUnit::Pixel => mouse_wheel_event.y * 15.,
        };
        if let Ok(mut scroll_position) = scrolled_node_query.single_mut() {
            scroll_position.offset_y -= dy;
        }
    }
}

pub fn initialize_grid_board(
    mut cmds: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    game_board: Res<BoardData>,
    font: Res<FontSpaceGrotesk>,
) {
    cmds.spawn((
        StateScoped(AppState::GameInProgress),
        TopBar,
        Node {
            width: Val::Percent(100.),
            height: Val::Px(CELL_SIZE),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        },
        BoxShadow::new(
            colors::DODGER_BLUE.with_alpha(0.5).into(),
            Val::Px(0.),
            Val::Px(-2.),
            Val::Px(2.),
            Val::Px(10.0),
        ),
    ))
    .with_children(|top_bar| {
        top_bar
            .spawn(Node {
                width: Val::Percent(50.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,

                ..default()
            })
            .with_children(|left_side| {
                let leave_button_style = UiButtonStyle {
                    color: colors::DODGER_BLUE.into(),
                    border_color: Color::WHITE,
                    text_color: Color::WHITE,
                };
                left_side
                    .spawn((
                        LeaveGameButton,
                        Button,
                        leave_button_style,
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(28.0),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            right: Val::Px(14.),
                            bottom: Val::Px(5.),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(4.0)),
                        BorderColor(leave_button_style.border_color),
                        BackgroundColor(leave_button_style.color),
                    ))
                    .with_child((
                        Text::new("Leave"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(leave_button_style.text_color),
                    ));

                left_side.spawn((
                    TurnOwnerLabel,
                    Label,
                    Text::new("X's Turn."),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(colors::GOLD.into()),
                ));
            });

        top_bar
            .spawn(Node {
                width: Val::Percent(50.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,

                ..default()
            })
            .with_children(|right_side| {
                right_side.spawn((
                    Label,
                    Text::new("Time Left:"),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(colors::GOLD.into()),
                ));
                right_side.spawn((
                    TurnTimeCounter,
                    Label,
                    Text::new(format!("{:.2}s", game_board.turn_duration)),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    });

    cmds.spawn((
        Grid,
        Node {
            width: Val::Px(CELL_SIZE * 3.),
            height: Val::Px(CELL_SIZE * 3.),
            top: Val::Px(244. - CELL_SIZE * 3.),
            left: Val::Px(4.),
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            row_gap: Val::Px(1.),
            column_gap: Val::Px(1.),
            display: Display::Grid,
            grid_template_rows: vec![RepeatedGridTrack::px(3, CELL_SIZE)],
            grid_template_columns: vec![RepeatedGridTrack::px(3, CELL_SIZE)],
            ..default()
        },
        ZIndex(0),
        BoxShadow::new(
            colors::GREEN_YELLOW.with_alpha(0.5).into(),
            Val::Px(0.),
            Val::Px(-2.),
            Val::Px(2.),
            Val::Px(10.0),
        ),
    ))
    .with_children(|grid| {
        for idx in 1u16..=9 {
            grid.spawn((
                GridCell(1 << (idx - 1)), // Stores each cell on a different bit
                Button,
                Node {
                    width: Val::Px(CELL_SIZE),
                    height: Val::Px(CELL_SIZE),
                    border: UiRect::all(Val::Px(2.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderColor(colors::GREEN_YELLOW.into()),
                BackgroundColor(colors::DODGER_BLUE.into()),
                BorderRadius::all(Val::Px(5.)),
            ));
        }
    });

    game_state.set(AppState::GameInProgress);
}

pub fn turn_expiration_time_update(
    mut turn_time_label_q: Query<&mut Text, With<TurnTimeCounter>>,
    mut game_board: ResMut<BoardData>,
    time: Res<Time>,
) -> Result {
    let mut turn_time_label = turn_time_label_q.single_mut()?;

    game_board.turn_duration -= time.delta_secs();
    *turn_time_label = format!("{:.2}s", game_board.turn_duration).into();
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn grid_cell_interaction(
    mut interaction_query: Query<
        (
            &GridCell,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>, Without<CellMarked>),
    >,
    board: Res<BoardData>,
    conn: Res<NetworkConnection>,
) {
    for (cell, interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if board.is_primary_turn() && board.cell_is_free(**cell) {
                    conn.reducers.mark_cell(board.id(), **cell).unwrap();
                }
            }
            Interaction::Hovered => {
                *color = colors::GREEN_YELLOW.into();
                *border_color = colors::DODGER_BLUE.into();
            }
            Interaction::None => {
                *color = colors::DODGER_BLUE.into();
                *border_color = colors::GREEN_YELLOW.into();
            }
        }
    }
}

pub fn clear_board(mut cmds: Commands, grid_q: Query<Entity, With<Grid>>) {
    if let Ok(grid) = grid_q.single() {
        cmds.entity(grid).despawn();
    }

    cmds.remove_resource::<BoardData>();
}

pub fn game_over_screen(
    mut cmds: Commands,
    font: Res<FontSpaceGrotesk>,
    board: Res<BoardData>,
    conn: Res<NetworkConnection>,
) {
    conn.reducers.leave_game(board.id()).unwrap();
    cmds.spawn((
        StateScoped(AppState::GameOverScreen),
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
            Text::new(board.result()),
            TextFont {
                font: font.clone(),
                font_size: 25.0,
                ..default()
            },
            TextColor(colors::GREEN_YELLOW.into()),
            BoxShadow::new(
                colors::DARK_VIOLET.into(),
                Val::Px(0.),
                Val::Px(0.),
                Val::Percent(60.),
                Val::Px(30.0),
            ),
        ));

        let go_back_button_style = UiButtonStyle {
            color: colors::GOLD.into(),
            border_color: colors::DEEP_PINK.into(),
            text_color: colors::DARK_VIOLET.into(),
        };

        parent
            .spawn((
                Button,
                go_back_button_style,
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Px(35.0),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(10.0)),
                BorderColor(go_back_button_style.border_color),
                BackgroundColor(go_back_button_style.color),
            ))
            .with_child((
                Text::new("Go Back"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(go_back_button_style.text_color),
            ));
    });
}

#[allow(clippy::type_complexity)]
pub fn game_over_screen_interaction(
    mut cmds: Commands,
    mut interaction_query: Query<
        (
            &UiButtonStyle,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor>,
    mut game_state: ResMut<NextState<AppState>>,
    board_systems: Res<BoardSystems>,
) {
    for (leave_button, interaction, mut color, mut border_color, children) in &mut interaction_query
    {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::WHITE.into();
                *border_color = leave_button.text_color.into();
                game_state.set(AppState::HomeScreen);
                cmds.run_system(board_systems["clear_board"]);
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

pub fn lobby_room_screen(mut cmds: Commands, font: Res<FontSpaceGrotesk>) {
    cmds.spawn((
        StateScoped(AppState::LobbyScreen),
        LobbyRoomScreen,
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
        parent
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BoxShadow::new(
                    colors::DODGER_BLUE.into(),
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Px(1.),
                    Val::Px(30.0),
                ),
            ))
            .with_children(|anchor| {
                anchor.spawn((
                    Label,
                    Text::new("Awaiting"),
                    TextFont {
                        font: font.clone(),
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(colors::GOLD.into()),
                ));
                anchor.spawn((
                    Label,
                    Text::new("opponnent"),
                    TextFont {
                        font: font.clone(),
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(colors::GOLD.into()),
                ));
                anchor.spawn((
                    Label,
                    Text::new("..."),
                    TextFont {
                        font: font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(colors::GOLD.into()),
                ));
            });

        let go_back_button_style = UiButtonStyle {
            color: colors::GOLD.into(),
            border_color: colors::DEEP_PINK.into(),
            text_color: colors::DARK_VIOLET.into(),
        };

        parent
            .spawn((
                Button,
                go_back_button_style,
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Px(35.0),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(10.0)),
                BorderColor(go_back_button_style.border_color),
                BackgroundColor(go_back_button_style.color),
            ))
            .with_child((
                Text::new("Leave"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(go_back_button_style.text_color),
            ));
    });
}

pub fn populate_lobby_from_cache(
    mut cmds: Commands,
    mut lobby_panel_q: Query<Entity, With<LobbyPanel>>,
    connection: Res<NetworkConnection>,
    font: Res<FontSpaceGrotesk>,
) -> bevy::prelude::Result {
    let lobby_entity = lobby_panel_q.single_mut()?;

    cmds.entity(lobby_entity).despawn_related::<Children>();
    cmds.entity(lobby_entity).with_children(|l| {
        for (idx, (room_id, game_id)) in connection
            .db()
            .lobby_room()
            .iter()
            .map(|r| (r.id, r.game_id.clone()))
            .enumerate()
        {
            let mut join_button = UiButtonStyle {
                color: colors::GREEN_YELLOW.into(),
                border_color: colors::DEEP_PINK.into(),
                text_color: colors::DARK_VIOLET.into(),
            };

            if idx % 2 != 0 {
                join_button.color = colors::GOLD.into();
            };

            l.spawn((
                LobbyRoomId(room_id),
                Node {
                    height: Val::Percent(95.),
                    width: Val::Percent(98.),
                    display: Display::Grid,
                    column_gap: Val::Px(2.),
                    overflow: Overflow::clip(),
                    grid_template_columns: vec![
                        RepeatedGridTrack::fr(1, 3.),
                        RepeatedGridTrack::fr(1, 2.),
                    ],
                    grid_auto_flow: GridAutoFlow::Column,
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    border: UiRect::bottom(Val::Px(2.)),
                    ..default()
                },
                BackgroundColor(colors::DARK_VIOLET.with_alpha(0.01).into()),
                BorderColor(colors::DODGER_BLUE.with_alpha(0.2).into()),
                BorderRadius::all(Val::Px(5.0)),
            ))
            .with_children(|lobby_row| {
                lobby_row.spawn((
                    Label,
                    Text::new(game_id.to_owned()),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(colors::GOLD.into()),
                ));
                lobby_row
                    .spawn((
                        JoinGameButton(room_id),
                        Button,
                        join_button,
                        Node {
                            height: Val::Percent(90.0),
                            min_width: Val::Px(50.),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            grid_column: GridPlacement::start(2),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(5.0)),
                        BorderColor(join_button.border_color),
                        BackgroundColor(join_button.color),
                    ))
                    .with_child((
                        Text::new("Join"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(join_button.text_color),
                    ));
            });
        }
    });
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn lobby_screen_leave_interaction(
    mut interaction_query: Query<
        (
            &UiButtonStyle,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor>,
    mut game_state: ResMut<NextState<AppState>>,
    conn: Res<NetworkConnection>,
) {
    for (leave_button, interaction, mut color, mut border_color, children) in &mut interaction_query
    {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::WHITE.into();
                *border_color = leave_button.text_color.into();
                conn.reducers.leave_room().unwrap();
                game_state.set(AppState::HomeScreen);
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

#[allow(clippy::type_complexity)]
pub fn leave_game_button_interaction(
    mut cmds: Commands,
    mut interaction_query: Query<
        (
            &UiButtonStyle,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<LeaveGameButton>),
    >,
    mut text_query: Query<&mut TextColor>,
    mut game_state: ResMut<NextState<AppState>>,
    board: Res<BoardData>,
    board_systems: Res<BoardSystems>,
    conn: Res<NetworkConnection>,
) {
    for (leave_button, interaction, mut color, mut border_color, children) in &mut interaction_query
    {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = Color::WHITE.into();
                *border_color = leave_button.text_color.into();
                conn.reducers.leave_game(board.id()).unwrap();
                game_state.set(AppState::HomeScreen);
                cmds.run_system(board_systems["clear_board"]);
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
