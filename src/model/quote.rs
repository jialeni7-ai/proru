#[derive(Clone, Debug)]
pub struct Quote {
    pub exchange:Exchange,
    pub bid_px: f64,
    pub bid_sz: f64,
    pub ask_px: f64,
    pub ask_sz: f64,
    pub ts: i64,
}

#[derive(Clone, Debug)]
pub enum Exchange {
    Okx,
    Bybit,
}