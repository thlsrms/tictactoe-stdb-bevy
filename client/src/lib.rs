mod network;
mod resources;
mod ui;

use bevy::prelude::*;

use resources::BoardSystems;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    Initialization,
    HomeScreen,
    LobbyScreen,
    GameSetup,
    GameInProgress,
    GameOverScreen,
}

pub struct TicTacToe;

impl Plugin for TicTacToe {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<BoardSystems>();

        app.init_state::<AppState>()
            .enable_state_scoped_entities::<AppState>();

        app.add_systems(
            OnEnter(AppState::Initialization),
            (resources::load_fonts, spawn_camera),
        );

        network::setup_systems(app);
        ui::setup_systems(app);
    }
}

fn spawn_camera(mut cmds: Commands) {
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
}
