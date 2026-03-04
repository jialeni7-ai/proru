use crate::model::quote::{Exchange, Quote};
use crate::model::signal::{Signal, State};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn strategy(okx: &Quote, bybit: &Quote, ct_mult: f64, state: &mut State) -> Option<Signal> {
    let now = now_ms();
    if now - okx.ts > 300 {
        return None;
    }
    if now - bybit.ts > 300 {
        return None;
    }
    if (okx.ts - bybit.ts).abs() > 80 {
        return None;
    }
    let entry_spread = (bybit.bid_px - okx.ask_px) / okx.ask_px;
    let exit_spread = (bybit.ask_px - okx.bid_px) / okx.bid_px;
    match *state {
        State::Idle => {
            if okx.ask_sz * ct_mult >= 2.0 && bybit.bid_sz >= 2.0 && entry_spread >= 0.005 {
                *state = State::Opened;
                return Some(Signal::OpenOkxLongBybitShort);
            }
        }
        State::Opened => {
            if exit_spread <= 0.0003 {
                *state = State::Idle;
                return Some(Signal::Close);
            }
        }
    }
    None
}
