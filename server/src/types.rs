use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Default, Clone, Copy, PartialEq)]
pub enum Player {
    #[default]
    X,
    O,
}

#[derive(SpacetimeType, Clone, PartialEq)]
pub enum GameState {
    InProgress,
    Draw,
    Winner(Player),
}
