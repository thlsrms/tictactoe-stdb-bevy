use std::time::Duration;

use spacetimedb::{ReducerContext, ScheduleAt, Table, TimeDuration, Timestamp};

use crate::game_table::game as _;

#[spacetimedb::table(name = game_duration_time_schedule, scheduled(scheduled_turn_expiration))]
pub struct GameDurationTimeSchedule {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: ScheduleAt,
    game_id: String,
    turn: u8,
}

#[spacetimedb::reducer]
pub fn scheduled_turn_expiration(
    ctx: &ReducerContext,
    arg: GameDurationTimeSchedule,
) -> Result<(), String> {
    if ctx.sender != ctx.identity() {
        return Err("Reducer `scheduled` may not be invoked by clients.".to_string());
    }
    let Some(mut game) = ctx.db.game().id().find(arg.game_id) else {
        return Ok(());
    };
    if game.turn != arg.turn {
        return Ok(());
    }
    game.turn_expired();
    set_turn_expiration_schedule(ctx, game.id.clone(), game.turn);
    ctx.db.game().id().update(game);
    Ok(())
}

pub fn set_turn_expiration_schedule(ctx: &ReducerContext, game_id: String, turn: u8) {
    let turn_time = TimeDuration::from_duration(Duration::from_secs_f32(duration_from_turn(turn)));
    let timestamp: Timestamp = ctx.timestamp + turn_time;
    ctx.db
        .game_duration_time_schedule()
        .insert(GameDurationTimeSchedule {
            scheduled_id: 0,
            scheduled_at: timestamp.into(),
            game_id,
            turn,
        });
}

fn duration_from_turn(n: u8) -> f32 {
    let decrement_1 = [1, 2, 4].iter().filter(|&&x| x <= n).count() as f32;
    let decrement_half = [6, 8].iter().filter(|&&x| x <= n).count() as f32;
    5.0 - decrement_1 - decrement_half * 0.5
}
