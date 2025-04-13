mod board_data;
mod tile;

use std::collections::HashMap;

use bevy::ecs::system::SystemId;
use bevy::prelude::*;

use board_data::BoardData;
use tile::{MarkTileEvent, Tile};

const TILE_SIZE: f32 = 60.;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Player {
    X,
    O,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum Game {
    #[default]
    Setup,
    InProgress,
    Over,
}

pub struct TicTacToe;

impl Plugin for TicTacToe {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<BoardSystems>();
        app.add_event::<MarkTileEvent>();
        app.init_state::<Game>();

        app.add_systems(OnEnter(Game::Setup), startup);
        app.add_systems(PostUpdate, register_mark.run_if(in_state(Game::InProgress)));
    }
}

fn startup(mut cmds: Commands, board_systems: Res<BoardSystems>) {
    cmds.run_system(board_systems["clear_board"]);
}

fn initialize_board(mut cmd: Commands, mut game_state: ResMut<NextState<Game>>) {
    cmd.spawn((
        Camera2d,
        Transform {
            translation: Vec3 {
                x: 90.,
                y: 120.,
                z: 1.,
            },
            ..Default::default()
        },
    ));
    for x in 0..=2 {
        for y in 0..=2 {
            Tile::spawn(
                &mut cmd,
                Vec2 {
                    x: (TILE_SIZE * x as f32) + TILE_SIZE / 2.,
                    y: (TILE_SIZE * y as f32) + TILE_SIZE / 2.,
                },
            );
        }
    }

    game_state.set(Game::InProgress);
}

fn register_mark(
    mut board: ResMut<BoardData>,
    mut ev_marked: EventReader<MarkTileEvent>,
    mut game_state: ResMut<NextState<Game>>,
) {
    if let Some(MarkTileEvent(cell_mask)) = ev_marked.read().next() {
        board.mark_cell(*cell_mask);
        if board.result().is_some() {
            game_state.set(Game::Over);
        }
    }
}

fn clear_board(
    mut cmds: Commands,
    tiles: Query<Entity, With<Tile>>,
    board_systems: Res<BoardSystems>,
) {
    for tile in &tiles {
        cmds.entity(tile).despawn();
    }

    cmds.insert_resource(BoardData::new());
    cmds.run_system(board_systems["initialize_board"]);
}

#[derive(Resource, Deref, DerefMut)]
struct BoardSystems(HashMap<String, SystemId>);

impl FromWorld for BoardSystems {
    fn from_world(world: &mut World) -> Self {
        let mut map = HashMap::new();

        map.insert(
            "initialize_board".into(),
            world.register_system(initialize_board),
        );
        map.insert("clear_board".into(), world.register_system(clear_board));

        Self(map)
    }
}
