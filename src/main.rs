use bevy::prelude::*;
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(188., 248.),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(tic_tac_toe::TicTacToe)
        .run();
}
