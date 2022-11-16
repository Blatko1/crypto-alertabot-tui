use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread, fmt::Display,
};

use binance::{errors::Result as BinanceResult, market::Market, model::SymbolPrice, api::Binance};

use crate::error::Result;

pub struct Bot {
    market: Arc<Market>,

    symbol: Symbol,
    price_tracker: PriceTracker,

    tick: u16,
}

// TODO replace unwrap() with ?
impl Bot {
    const TICKS_PER_UPDATE: u16 = 2;
    const DEFAULT_SYMBOL: &str = "ETHUSDT";

    pub fn with_symbol<S: Into<Symbol>>(symbol: S) -> Result<Self> {
        let symbol = symbol.into();
        let market = Arc::new(Market::new(None, None));
        let price_tracker = PriceTracker::new(market.clone(), symbol);
        Ok(Self {
            market,

            symbol,
            price_tracker,

            tick: 0,
        })
    }

    pub fn new() -> Result<Self> {
        Self::with_symbol(Self::DEFAULT_SYMBOL)
    }

    pub fn analyze(&mut self) {}

    pub fn tick(&mut self) {
        self.price_tracker.track();
        self.tick += 1;

        if self.tick >= Self::TICKS_PER_UPDATE {
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
            match price {
                Ok(p) => self.price = p.into(),
                Err(err) => println!("Binance Error: {err}"),
            }
            println!("price: {}", self.price);
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

impl Display for PriceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Price: {}", self.0)
    }
}

impl Into<PriceLevel> for SymbolPrice {
    fn into(self) -> PriceLevel {
        PriceLevel(self.price)
    }
}
