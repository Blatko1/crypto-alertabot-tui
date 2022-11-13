use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread,
};

use binance::{api::Binance, errors::Result as BinanceResult, market::Market, model::SymbolPrice};

pub struct MarketBot {
    market: Arc<Market>,
    symbol: Symbol,
    price_tracker: PriceTracker,

    tick: u16,
}

// TODO replace unwrap() with ?
impl MarketBot {
    const UPDATE_TICKS: u16 = 5;

    pub fn new<S: Into<Symbol>>(symbol: S, zones: Vec<Zone>) -> Self {
        let symbol = symbol.into();
        let market = Arc::new(Market::new(None, None));
        let price_tracker = PriceTracker::new(market.clone(), symbol);
        Self {
            market: market,
            symbol,
            price_tracker,

            tick: 0,
        }
    }

    pub fn analyze(&mut self) {}

    pub fn tick(&mut self) {
        self.price_tracker.track();
        self.tick += 1;

        if self.tick >= Self::UPDATE_TICKS {
            self.analyze();
            self.tick = 0;
        }
    }

    // TODO maybe do inlining
    pub fn get_price(&self) -> PriceLevel {
        self.price_tracker.get_price()
    }

    pub fn get_symbol(&self) -> Symbol {
        self.symbol
    }
}

struct PriceTracker {
    price: PriceLevel,
    reader: Receiver<BinanceResult<SymbolPrice>>,
}

impl PriceTracker {
    fn new(market: Arc<Market>, symbol: Symbol) -> Self {
        let reader = Self::spawn_price_reader(market, symbol);
        Self {
            price: PriceLevel::NAN,
            reader,
        }
    }

    fn track(&mut self) {
        if let Some(price) = self.reader.try_iter().last() {
            // TODO add Into impl for PriceLevel from SymbolPrice
            self.price = price.unwrap().into();
        }
    }

    fn get_price(&self) -> PriceLevel {
        self.price
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
    ) -> Receiver<BinanceResult<SymbolPrice>> {
        let (tx, rx) = channel();

        thread::spawn(move || loop {
            let price = market.get_price(symbol);
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

#[derive(Debug, Clone, Copy)]
pub struct PriceLevel(pub f64);

impl PriceLevel {
    pub const NAN: PriceLevel = PriceLevel(f64::NAN);
}

impl Into<PriceLevel> for SymbolPrice {
    fn into(self) -> PriceLevel {
        PriceLevel(self.price)
    }
}
