use bevy::prelude::*;

use crate::board_data::BoardData;
use crate::{Game, Player, TILE_SIZE};

#[derive(Event)]
pub struct MarkTileEvent(pub u16);

#[derive(Component)]
struct TileMarked;

#[derive(Component)]
pub struct Tile;

impl Tile {
    pub fn spawn(cmd: &mut Commands, Vec2 { x, y }: Vec2) {
        cmd.spawn((
            Sprite {
                color: Color::srgba_u8(128, 128, 128, 255),
                custom_size: Some(Vec2 { x: 58., y: 58. }),
                ..Default::default()
            },
            Transform {
                translation: Vec3 { x, y, z: 1. },
                ..Default::default()
            },
            Tile,
        ))
        .observe(Self::event_hover_in)
        .observe(Self::event_hover_out)
        .observe(Self::event_click);
    }

    fn event_hover_in(
        hover_in: Trigger<Pointer<Over>>,
        mut tiles: Query<&mut Sprite, (With<Tile>, Without<TileMarked>)>,
    ) {
        let Ok(mut tile) = tiles.get_mut(hover_in.entity()) else {
            return;
        };
        tile.color = Color::srgba_u8(255, 255, 255, 255);
    }

    fn event_hover_out(
        hover_out: Trigger<Pointer<Out>>,
        mut tiles: Query<&mut Sprite, (With<Tile>, Without<TileMarked>)>,
    ) {
        let Ok(mut tile) = tiles.get_mut(hover_out.entity()) else {
            return;
        };
        tile.color = Color::srgba_u8(128, 128, 128, 255);
    }

    fn event_click(
        clicked: Trigger<Pointer<Click>>,
        mut cmds: Commands,
        board: Res<BoardData>,
        game_state: Res<State<Game>>,
        mut tiles: Query<(&mut Sprite, &Transform), With<Tile>>,
        mut ev_mark_tile: EventWriter<MarkTileEvent>,
    ) {
        if *game_state.get() != Game::InProgress {
            return;
        }
        let Ok((mut tile, xform)) = tiles.get_mut(clicked.entity()) else {
            return;
        };

        let (x, y) = Self::xform_to_grid(xform.translation.x, xform.translation.y);
        if board.cell_is_free(x, y) {
            match board.turn_owner() {
                Player::X => {
                    tile.color = Color::srgba_u8(255, 51, 0, 255);
                }
                Player::O => {
                    tile.color = Color::srgba_u8(0, 125, 255, 255);
                }
            }
            cmds.entity(clicked.entity()).insert(TileMarked);
            cmds.entity(clicked.observer()).despawn();
            ev_mark_tile.send(MarkTileEvent(BoardData::coord_to_bit(x, y)));
        }
    }

    fn xform_to_grid(x: f32, y: f32) -> (i32, i32) {
        let x = (x / TILE_SIZE).floor() as i32;
        let y = (y / TILE_SIZE).floor() as i32;
        (x, y)
    }
}
