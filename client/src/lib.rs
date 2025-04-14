mod board_data;
mod cell;
mod colors;
mod ui;

use std::collections::HashMap;

use bevy::ecs::system::SystemId;
use bevy::prelude::*;

use board_data::BoardData;
use cell::{Cell, CellMarkEvent, Grid, grid_cell_interaction, mark_cell};
use ui::{FontAsset, FontSpaceGrotesk};

const CELL_SIZE: f32 = 60.;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Player {
    X,
    O,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Initialization,
    Idle,
    Start,
    InProgress,
    Over,
}

pub struct TicTacToe;

impl Plugin for TicTacToe {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<BoardSystems>();
        app.add_event::<CellMarkEvent>();
        app.init_state::<GameState>();

        app.add_systems(
            OnEnter(GameState::Initialization),
            (load_assets, spawn_camera).chain(),
        );

        app.add_systems(OnEnter(GameState::Idle), ui::home_screen);
        app.add_systems(
            Update,
            ui::home_screen_interaction.run_if(in_state(GameState::Idle)),
        );
        app.add_systems(OnExit(GameState::Idle), ui::home_screen_cleanup);

        app.add_systems(OnEnter(GameState::Start), initialize_board);

        app.add_systems(
            Update,
            grid_cell_interaction.run_if(in_state(GameState::InProgress)),
        );
        app.add_systems(
            PostUpdate,
            mark_cell.run_if(in_state(GameState::InProgress)),
        );

        app.add_systems(OnEnter(GameState::Over), ui::game_over_screen);
        app.add_systems(
            Update,
            ui::game_over_screen_interaction.run_if(in_state(GameState::Over)),
        );
        app.add_systems(OnExit(GameState::Over), ui::game_over_screen_cleanup);
    }
}

fn load_assets(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load::<Font>(FontAsset::SpaceGrotesk.as_ref());
    cmds.insert_resource(FontSpaceGrotesk(font_handle));
}

fn spawn_camera(mut cmds: Commands, mut game_state: ResMut<NextState<GameState>>) {
    cmds.spawn((
        Camera2d,
        Transform {
            translation: Vec3 {
                x: 90.,
                y: 120.,
                z: 1.,
            },
            ..default()
        },
    ));

    game_state.set(GameState::Idle);
}

fn initialize_board(mut cmds: Commands, mut game_state: ResMut<NextState<GameState>>) {
    cmds.insert_resource(BoardData::new());
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
    ))
    .with_children(|grid| {
        for idx in 1u16..=9 {
            grid.spawn((
                Cell(1 << (idx - 1)), // Stores each cell on a different bit
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

    game_state.set(GameState::InProgress);
}

fn clear_board(mut cmds: Commands, grid: Query<Entity, With<Grid>>) {
    if let Ok(grid) = grid.get_single() {
        cmds.entity(grid).despawn_recursive();
    }

    cmds.remove_resource::<BoardData>();
}

#[derive(Resource, Deref, DerefMut)]
struct BoardSystems(HashMap<String, SystemId>);

impl FromWorld for BoardSystems {
    fn from_world(world: &mut World) -> Self {
        let mut map = HashMap::new();
        map.insert("clear_board".into(), world.register_system(clear_board));
        Self(map)
    }
}
