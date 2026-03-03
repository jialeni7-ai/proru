#[derive(Clone, Debug)]
pub struct Quote {
    pub exchange:Exchange,
    pub bid_px: String,
    pub bid_sz: String,
    pub ask_px: String,
    pub ask_sz: String,
    pub ts: i64,
}

#[derive(Clone, Debug)]
pub enum Exchange {
    Okx,
    Bybit,
}