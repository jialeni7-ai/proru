use crate::model::quote::Quote;
use crate::model::signal::{Signal, State};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn strategy(okx: &Quote, bybit: &Quote, ct_mult: f64, state: &State) -> Option<Signal> {
    let now = now_ms();
    if now - okx.ts > 300 {
        return None;
    }
    if now - bybit.ts > 300 {
        return None;
    }
    if okx.ts.abs_diff(bybit.ts) > 80 {
        return None;
    }
    let entry_spread = (bybit.bid_px - okx.ask_px) / okx.ask_px;
    // let entry_spread = (okx.bid_px - bybit.ask_px) / bybit.ask_px;
    // println!("{}",entry_spread);
    let exit_spread = (bybit.ask_px - okx.bid_px) / okx.bid_px;
    match *state {
        State::Idle => {
            if okx.bid_sz * ct_mult >= 10.0 && bybit.ask_sz >= 10.0 && entry_spread >= 0.006 {
                return Some(Signal::OpenOkxLongBybitShort);
            }
        }
        State::Opened => {
            if exit_spread <= 0.0003 {
                return Some(Signal::Close);
            }
        }
    }
    None
}
