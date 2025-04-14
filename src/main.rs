use bevy::asset::AssetMetaCheck;
use bevy::color::palettes::css::BLUE_VIOLET;
use bevy::prelude::*;
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
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
                }),
        )
        //.insert_resource(ClearColor(Color::srgb(0.2, 0.21, 0.27)))
        .insert_resource(ClearColor(BLUE_VIOLET.into()))
        .add_plugins(tic_tac_toe::TicTacToe)
        .run();
}
