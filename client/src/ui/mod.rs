pub mod colors;
mod systems;

use bevy::prelude::*;

pub use systems::*;

#[derive(Component)]
pub struct HomeScreen;

#[derive(Component)]
pub struct LobbyPanel;

#[derive(Component)]
pub struct LobbyRoomId(pub u32);

#[derive(Component)]
pub struct JoinGameButton(pub u32);

#[derive(Component)]
pub struct NewGameButton;

#[derive(Component, Clone, Copy)]
pub struct UiButtonStyle {
    pub color: Color,
    pub border_color: Color,
    pub text_color: Color,
}

#[derive(Component)]
struct TopBar;

#[derive(Component)]
pub struct TurnOwnerLabel;

#[derive(Component)]
pub struct TurnTimeCounter;

#[derive(Component)]
pub struct LeaveGameButton;

const CELL_SIZE: f32 = 60.;

#[derive(Component)]
pub struct CellMarked;

#[derive(Component, Deref)]
pub struct GridCell(pub u16);

#[derive(Component)]
pub struct Grid;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct LobbyRoomScreen;
