use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread, fmt::Display,
};

use binance::{errors::Result as BinanceResult, market::Market, model::{SymbolPrice, PriceStats}, api::Binance};

use crate::error::Result;

pub struct Bot {
    market: Arc<Market>,

    symbol: Symbol,
    live_stats: LiveStats,

    tick: u16,
}

impl Bot {
    const TICKS_PER_UPDATE: u16 = 2;
    const DEFAULT_SYMBOL: &str = "ETHUSDT";

    pub fn with_symbol<S: Into<Symbol>>(symbol: S) -> Result<Self> {
        let symbol = symbol.into();
        let market = Arc::new(Market::new(None, None));
        let live_stats = LiveStats::new(market.clone(), symbol);
        Ok(Self {
            market,

            symbol,
            live_stats,

            tick: 0,
        })
    }

    pub fn new() -> Result<Self> {
        Self::with_symbol(Self::DEFAULT_SYMBOL)
    }

    pub fn analyze(&mut self) {}

    pub fn tick(&mut self) {
        self.live_stats.update();
        self.tick += 1;

        if self.tick >= Self::TICKS_PER_UPDATE {
            self.analyze();
            self.tick = 0;
        }
    }

    // TODO maybe do inlining
    pub fn get_price(&self) -> PriceLevel {
        self.live_stats.price()
    }

    pub fn get_symbol(&self) -> Symbol {
        self.symbol
    }
}

struct LiveStats {
    last_price: PriceLevel,
    price_change: String,
    price_change_percent: String,
    volume: f64,
    reader: Receiver<BinanceResult<PriceStats>>,
}

impl LiveStats {
    fn new(market: Arc<Market>, symbol: Symbol) -> Self {
        let reader = Self::spawn_price_reader(market, symbol);
        Self {
            last_price: Default::default(),
            price_change: String::from("{PRICE CHANGE}"),
            price_change_percent: String::from("{PRICE CHANGE PERCENT}"),
            volume: Default::default(),
            reader,
        }
    }

    fn update(&mut self) {
        if let Some(price) = self.reader.try_iter().last() {
            match price {
                Ok(stats) => {self.stats = Some(stats)},
                Err(err) => println!("Binance Error: {err}"),
            }
        }
    }

    pub fn last_price(&self) -> PriceLevel {
        self.last_price
    }

    /// Reading the price from Binance charts blocks the thread for a short period of time
    /// which can sometimes delay the user input so a new thread is needed.
    ///
    /// Every [`crate::TICK_INTERVAL`] this thread reads the market price and sends it to
    /// the main thread which stores it in the next [`crate::TICK_INTERVAL`].
    ///
    /// If the price reader thread looses connection with the main thread it will just exit
    /// and the main thread will probably just spawn a new one.
    fn spawn_price_reader(
        market: Arc<Market>,
        symbol: Symbol,
    ) -> Receiver<BinanceResult<PriceStats>> {
        let (tx, rx) = channel();
        thread::spawn(move || loop {
            let price = market.get_24h_price_stats(symbol);
            //market.get_klines(symbol, "1m", None, None, None)
            match tx.send(price) {
                Ok(_) => thread::sleep(crate::TICK_INTERVAL),
                Err(_) => break,
            }
        });
        rx
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Symbol(pub &'static str);

impl Into<Symbol> for &'static str {
    fn into(self) -> Symbol {
        Symbol(self)
    }
}

impl Into<String> for Symbol {
    fn into(self) -> String {
        self.0.to_owned()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PriceLevel(pub f64);

impl PriceLevel {
    pub const NAN: PriceLevel = PriceLevel(f64::NAN);
}

impl Display for PriceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Price: {}", self.0)
    }
}