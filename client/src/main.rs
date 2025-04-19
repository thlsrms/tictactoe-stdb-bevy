use bevy::asset::{AssetMetaCheck, embedded_asset};
use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tic-Tac-Toe".into(),
                    resolution: WindowResolution::new(188., 248.),
                    resizable: false,
                    ..default()
                }),
                close_when_requested: true,
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(LogPlugin {
                filter: "wgpu_core=warn,wgpu_hal=warn,tic_tac_toe=debug".into(),
                level: log::Level::WARN,
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::BLACK.with_alpha(0.975)))
    .add_plugins(tic_tac_toe::TicTacToe);

    embedded_asset!(app, "src/", "../assets/fonts/SpaceGrotesk-Medium.ttf");

    app.run();
}
