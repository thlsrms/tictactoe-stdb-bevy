use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::tasks::futures_lite;
use spacetimedb_sdk::DbContext as _;

use crate::AppState;
use crate::resources::{BoardData, FontSpaceGrotesk};
use crate::ui::{
    CellMarked, GridCell, JoinGameButton, LobbyPanel, LobbyRoomId, TurnOwnerLabel, UiButtonStyle,
    colors,
};

use super::{
    Game, GameState, LobbyRoom, NetworkAuth, NetworkConnection, NetworkEvent, OnConnect, OnDelete,
    OnInsert, OnUpdate, Player,
};

pub fn setup_systems(app: &mut App) {
    super::connect_stdb(app);

    let update_initialization = on_network_connected.run_if(in_state(AppState::Initialization));

    let update_home_screen = (
        on_lobby_room_created.run_if(on_event::<NetworkEvent<OnInsert<LobbyRoom>>>),
        on_lobby_room_removed.run_if(on_event::<NetworkEvent<OnDelete<LobbyRoom>>>),
        on_game_created.run_if(on_event::<NetworkEvent<OnInsert<Game>>>),
    )
        .run_if(in_state(AppState::HomeScreen));

    let update_lobby_sceen = (on_game_created.run_if(on_event::<NetworkEvent<OnInsert<Game>>>),)
        .run_if(in_state(AppState::LobbyScreen));

    let update_game_in_progress = (
        on_game_updated.run_if(on_event::<NetworkEvent<OnUpdate<Game>>>),
        on_game_deleted.run_if(on_event::<NetworkEvent<OnDelete<Game>>>),
    )
        .run_if(in_state(AppState::GameInProgress));

    #[cfg(target_arch = "wasm32")]
    app.add_systems(
        PreUpdate,
        poll_pending_connection.run_if(in_state(AppState::Initialization)),
    );

    app.add_systems(
        Update,
        (
            update_initialization,
            update_home_screen,
            update_lobby_sceen,
            update_game_in_progress,
        ),
    );
}

#[cfg(target_arch = "wasm32")]
fn poll_pending_connection(
    mut cmds: Commands,
    maybe_pending: Option<ResMut<super::PendingConnection>>,
) {
    let Some(mut pending) = maybe_pending else {
        return;
    };

    if let Some(mut conn) =
        futures_lite::future::block_on(futures_lite::future::poll_once(&mut pending.0))
    {
        super::register_callbacks(&mut cmds, &mut conn);
        conn.run_threaded();
        cmds.insert_resource(NetworkConnection::new(conn));
        cmds.remove_resource::<super::PendingConnection>();
    }
}

pub fn on_network_connected(
    mut cmds: Commands,
    mut on_connected_ev: EventReader<NetworkEvent<OnConnect>>,
    maybe_connection: Option<Res<NetworkConnection>>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    let Some(connection) = maybe_connection else {
        return;
    };
    for NetworkEvent(OnConnect(NetworkAuth {
        identity,
        authorization,
    })) in on_connected_ev.read()
    {
        info!("Connection established");
        info!("Identity '{identity}'");

        #[cfg(target_arch = "wasm32")]
        gloo_console::log!("Connection established");
        #[cfg(target_arch = "wasm32")]
        gloo_console::log!("Identity '", format!("{identity}'"));

        let _ = connection.subscription_builder().subscribe(format!(
            "SELECT * FROM game WHERE x_player = '{}' OR o_player = '{}'",
            identity, identity
        ));

        #[cfg(target_arch = "wasm32")]
        {
            // On the web we can't connect using `with_token`, so we would have
            // to send our authorization token through a reducer if needed.
            let authorization: String = {
                let creds = spacetimedb_sdk::credentials::StorageEntry::new("tic-tac-toe_token");
                if creds.load().as_ref().is_ok_and(|t| t.is_none()) {
                    _ = creds.save(authorization.clone());
                    authorization.to_owned()
                } else if let Ok(Some(token)) = creds.load() {
                    token
                } else {
                    _ = creds.save(authorization.clone());
                    authorization.to_owned()
                }
            };

            cmds.insert_resource(NetworkAuth {
                identity: *identity,
                authorization: authorization.to_owned(),
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        cmds.insert_resource(NetworkAuth {
            identity: *identity,
            authorization: authorization.to_owned(),
        });

        // Connection established, we can load the home_screen
        game_state.set(AppState::HomeScreen);
    }
}

pub fn on_lobby_room_created(
    mut cmds: Commands,
    mut new_lobby_room_ev: EventReader<NetworkEvent<OnInsert<LobbyRoom>>>,
    mut lobby_panel_q: Query<(Entity, &mut ScrollPosition), With<LobbyPanel>>,
    lobby_rooms_q: Query<&Children, With<LobbyPanel>>,
    font: Res<FontSpaceGrotesk>,
    network_auth: Res<NetworkAuth>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    let Ok((lobby_entity, mut scroll_position)) = lobby_panel_q.get_single_mut() else {
        warn!("LobbyRoom new room early return! No LobbyPanel!");
        return;
    };
    let Some(new_room) = new_lobby_room_ev.read().next() else {
        warn!("LobbyRoom new room early return! System triggered without event?");
        return;
    };

    if network_auth.identity == new_room.owner {
        game_state.set(AppState::LobbyScreen);
        return;
    }

    let (idx, room_id, game_id) = (
        lobby_rooms_q.iter().count(),
        new_room.id,
        new_room.game_id.clone(),
    );

    let mut join_button = UiButtonStyle {
        color: colors::GREEN_YELLOW.into(),
        border_color: colors::DEEP_PINK.into(),
        text_color: colors::DARK_VIOLET.into(),
    };
    if idx % 2 != 0 {
        join_button.color = colors::GOLD.into();
    };

    let lobby_room = cmds
        .spawn((
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
                Text::new(game_id),
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
        })
        .id();
    cmds.entity(lobby_entity).add_child(lobby_room);

    scroll_position.offset_y = 100000.;
}

pub fn on_lobby_room_removed(
    mut cmds: Commands,
    mut lobby_room_del_ev: EventReader<NetworkEvent<OnDelete<LobbyRoom>>>,
    lobby_rooms_q: Query<(Entity, &LobbyRoomId)>,
) {
    let Some(room_removed) = lobby_room_del_ev.read().next() else {
        warn!("LobbyRoom room removed early return! NO EVENnt?");
        return;
    };

    if let Some((id, _)) = lobby_rooms_q
        .iter()
        .find(|(_, room_id)| room_id.0 == room_removed.id)
    {
        cmds.entity(id).despawn_recursive();
    }
}

pub fn on_game_created(
    mut cmds: Commands,
    mut game_created_ev: EventReader<NetworkEvent<OnInsert<Game>>>,
    network_auth: Res<NetworkAuth>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    let Some(game) = game_created_ev.read().next() else {
        warn!("Game Created: none early return");
        return;
    };

    let board_data = {
        if network_auth.identity == game.x_player {
            BoardData::new(Player::X, game.id.clone())
        } else {
            BoardData::new(Player::O, game.id.clone())
        }
    };
    cmds.insert_resource(board_data);
    game_state.set(AppState::GameSetup);
}

pub fn on_game_deleted(
    mut game_deleted_ev: EventReader<NetworkEvent<OnDelete<Game>>>,
    mut game_board: ResMut<BoardData>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    let Some(_game) = game_deleted_ev.read().next() else {
        warn!("Game Deleted: none early return");
        return;
    };

    // If this client received the on_delete event we assume the other client disconnected
    game_board.set_result_network_primary();
    game_state.set(AppState::GameOverScreen);
}

pub fn on_game_updated(
    mut cmds: Commands,
    mut cell_q: Query<(Entity, &mut BackgroundColor, &GridCell), Without<CellMarked>>,
    mut turn_owner_label_q: Query<(&mut Text, &mut TextColor), With<TurnOwnerLabel>>,
    mut game_update_ev: EventReader<NetworkEvent<OnUpdate<Game>>>,
    mut game_board: ResMut<BoardData>,
    mut game_state: ResMut<NextState<AppState>>,
    font: Res<FontSpaceGrotesk>,
) {
    let Some(NetworkEvent(OnUpdate { old, new })) = game_update_ev.read().next() else {
        warn!("Game Updated: none early return");
        return;
    };

    if (old.turn_owner != new.turn_owner) && !new.time_expired {
        // No time_expired, we assume the old turn_owner has made a valid move to trigger the update
        let (cell_marked_id, letter, bg_color, text_color) = match old.turn_owner {
            Player::X => {
                game_board.x_mask = new.x_mask;
                (
                    old.x_mask ^ new.x_mask,
                    "X",
                    colors::GOLD.into(),
                    colors::DARK_VIOLET.into(),
                )
            }
            Player::O => {
                game_board.o_mask = new.o_mask;
                (
                    old.o_mask ^ new.o_mask,
                    "O",
                    colors::DEEP_PINK.into(),
                    colors::GREEN_YELLOW.into(),
                )
            }
        };

        let Some((entity_id, mut color, _)) =
            cell_q.iter_mut().find(|(_, _, c)| c.0 == cell_marked_id)
        else {
            warn!("CellEntity '{cell_marked_id}' modified but was not found");
            return;
        };
        *color = bg_color;
        let mut e = cmds.entity(entity_id);
        e.insert(CellMarked);
        e.with_child((
            Text::new(letter),
            TextFont {
                font: font.clone(),
                font_size: 40.0,
                ..default()
            },
            TextColor(text_color),
        ));
    }

    match new.state {
        GameState::InProgress => {
            game_board.turn_duration = duration_from_turn(new.turn);
            game_board.turn_owner = new.turn_owner;

            let (mut turn_owner_label, mut text_color) = turn_owner_label_q.single_mut();
            *turn_owner_label = {
                if game_board.turn_owner == game_board.network_primary {
                    *text_color = Color::WHITE.into();
                    "Your Turn!"
                } else {
                    *text_color = colors::GOLD.into();
                    match game_board.turn_owner {
                        Player::X => "X's Turn.",
                        Player::O => "O's Turn",
                    }
                }
            }
            .into();
        }
        GameState::Draw => {
            game_board.set_result_draw();
            game_state.set(AppState::GameOverScreen);
        }
        GameState::Winner(player) => {
            game_board.set_result_winner(&player);
            game_state.set(AppState::GameOverScreen);
        }
    }
}

fn duration_from_turn(n: u8) -> f32 {
    let decrement_1 = [1, 2, 4].iter().filter(|&&x| x <= n).count() as f32;
    let decrement_half = [6, 8].iter().filter(|&&x| x <= n).count() as f32;
    5.0 - decrement_1 - decrement_half * 0.5
}
