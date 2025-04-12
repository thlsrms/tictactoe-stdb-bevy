use bevy::prelude::*;

pub struct TicTacToe;

impl Plugin for TicTacToe {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(GameState::new())
            .add_event::<MarkTileEvent>()
            .add_systems(Startup, initialize_board)
            .add_systems(PostUpdate, register_mark);
    }
}

const TILE_SIZE: f32 = 60.;

fn initialize_board(mut cmd: Commands) {
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
}

/*
* Bitboard:
* Map each board cell to one of nine bits (positions 0 to 8):
* (0,0) -> bit 0, (1,0) -> bit 1, (2,0) -> bit 2, (0,1) -> bit 3, ..., (2,2) -> bit 8.
*/
const WINNING_MASKS: [u16; 8] = [
    0b000_000_111, // row 0
    0b000_111_000, // row 1
    0b111_000_000, // row 2
    0b001_001_001, // col 0
    0b010_010_010, // col 1
    0b100_100_100, // col 2
    0b100_010_001, // main diagonal
    0b001_010_100, // anti-diagonal
];

#[derive(Resource)]
struct GameState {
    turn_owner: Player,
    x_mask: u16,
    o_mask: u16,
    result: GameResult,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            turn_owner: Player::X,
            x_mask: 0,
            o_mask: 0,
            result: GameResult::InProgress,
        }
    }

    pub fn turn_owner(&self) -> Player {
        self.turn_owner
    }

    pub fn result(&self) -> GameResult {
        self.result
    }

    pub fn next_turn(&mut self) {
        self.turn_owner = match self.turn_owner {
            Player::X => Player::O,
            Player::O => Player::X,
        };
    }

    pub fn mark_cell(&mut self, cell_mask: u16) {
        match self.turn_owner {
            Player::X => self.x_mask |= cell_mask,
            Player::O => self.o_mask |= cell_mask,
        }
        self.check_winner();
    }

    /// Check if the current player has a winning combination.
    /// Uses precomputed winning masks to determine a win.
    pub fn check_winner(&mut self) {
        let curr_player_mask = {
            match self.turn_owner {
                Player::X => self.x_mask,
                Player::O => self.o_mask,
            }
        };

        let match_won = WINNING_MASKS
            .iter()
            .any(|&mask| (curr_player_mask & mask) == mask);

        if match_won {
            self.result = GameResult::Winner(self.turn_owner());
            info!("Match over. {:?} won!", self.turn_owner);
        } else if self.has_potential_win() {
            self.next_turn();
        } else {
            self.result = GameResult::Draw;
            info!("Match over. It's a DRAW");
        }
    }

    /// Converts (x, y) coordinates to a bit index.
    fn coord_to_bit(x: i32, y: i32) -> u16 {
        1 << (x + y * 3)
    }

    /// Check if there's any line still potentially open for a win.
    fn has_potential_win(&self) -> bool {
        for &mask in &WINNING_MASKS {
            // The mask is blocked if both players have occupied a cell in the mask
            if !((mask & self.x_mask != 0) && (mask & self.o_mask != 0)) {
                return true;
            }
        }
        false
    }

    pub fn cell_is_free(&self, x: i32, y: i32) -> bool {
        let cell_mask = Self::coord_to_bit(x, y);
        (self.x_mask & cell_mask == 0) && (self.o_mask & cell_mask == 0)
    }
}

#[derive(Event)]
struct MarkTileEvent(u16);

#[derive(Clone, Copy, PartialEq, Debug)]
enum GameResult {
    InProgress,
    Draw,
    Winner(Player),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Player {
    X,
    O,
}

#[derive(Component)]
struct TileMarked;

#[derive(Component)]
struct Tile;

impl Tile {
    fn spawn(cmd: &mut Commands, Vec2 { x, y }: Vec2) {
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
        state: Res<GameState>,
        mut tiles: Query<(&mut Sprite, &Transform), With<Tile>>,
        mut ev_mark_tile: EventWriter<MarkTileEvent>,
    ) {
        if state.result() != GameResult::InProgress {
            return;
        }
        let Ok((mut tile, xform)) = tiles.get_mut(clicked.entity()) else {
            return;
        };

        let (x, y) = Self::xform_to_grid(xform.translation.x, xform.translation.y);
        if state.cell_is_free(x, y) {
            match state.turn_owner() {
                Player::X => {
                    tile.color = Color::srgba_u8(255, 51, 0, 255);
                }
                Player::O => {
                    tile.color = Color::srgba_u8(0, 125, 255, 255);
                }
            }
            cmds.entity(clicked.entity()).insert(TileMarked);
            cmds.entity(clicked.observer()).despawn();
            ev_mark_tile.send(MarkTileEvent(GameState::coord_to_bit(x, y)));
        }
    }

    fn xform_to_grid(x: f32, y: f32) -> (i32, i32) {
        let x = (x / TILE_SIZE).floor() as i32;
        let y = (y / TILE_SIZE).floor() as i32;
        (x, y)
    }
}

fn register_mark(mut state: ResMut<GameState>, mut ev_marked: EventReader<MarkTileEvent>) {
    if let Some(MarkTileEvent(cell_mask)) = ev_marked.read().next() {
        state.mark_cell(*cell_mask);
    }
}
