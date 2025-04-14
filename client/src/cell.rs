use bevy::prelude::*;

use crate::board_data::BoardData;
use crate::colors;
use crate::ui::FontSpaceGrotesk;
use crate::{GameState, Player};

#[derive(Event)]
pub struct CellMarkEvent(pub u16);

#[derive(Component)]
pub struct CellMarked;

#[derive(Component, Deref)]
pub struct Cell(pub u16);

#[derive(Component)]
pub struct Grid;

#[allow(clippy::type_complexity)]
pub fn grid_cell_interaction(
    mut cmds: Commands,
    mut interaction_query: Query<
        (
            &Cell,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Entity,
        ),
        (Changed<Interaction>, With<Button>, Without<CellMarked>),
    >,
    board: Res<BoardData>,
    mut ev_mark_cell: EventWriter<CellMarkEvent>,
    font: Res<FontSpaceGrotesk>,
) {
    for (cell, interaction, mut color, mut border_color, entity) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if board.cell_is_free(**cell) {
                    let mut e = cmds.entity(entity);
                    e.insert(CellMarked);
                    let (letter, color) = match board.turn_owner() {
                        Player::X => {
                            *color = colors::GOLD.into();
                            ("X", colors::DARK_VIOLET.into())
                        }
                        Player::O => {
                            *color = colors::DEEP_PINK.into();
                            ("O", colors::GREEN_YELLOW.into())
                        }
                    };
                    e.with_child((
                        Text::new(letter),
                        TextFont {
                            font: font.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(color),
                    ));
                    ev_mark_cell.send(CellMarkEvent(**cell));
                }
            }
            Interaction::Hovered => {
                *color = colors::GREEN_YELLOW.into();
                *border_color = colors::DODGER_BLUE.into();
            }
            Interaction::None => {
                *color = colors::DODGER_BLUE.into();
                *border_color = colors::GREEN_YELLOW.into();
            }
        }
    }
}

pub fn mark_cell(
    mut board: ResMut<BoardData>,
    mut ev_marked: EventReader<CellMarkEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Some(CellMarkEvent(cell_mask)) = ev_marked.read().next() {
        board.mark_cell(*cell_mask);
        if board.result().is_some() {
            game_state.set(GameState::Over);
        }
    }
}
