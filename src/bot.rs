use std::{
    fmt::Display,
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread,
};

use binance::{api::Binance, errors::Result as BinanceResult, market::Market, model::PriceStats};

use crate::error::Result;

pub struct Bot {
    market: Arc<Market>,

    symbol: Symbol,
    live_stats_tracker: LiveStatsTracker,

    price_triggers: Vec<PriceTrigger>,
    latest_alerts: Vec<Alert>,

    //tick: u16,
}

impl Bot {
    //const TICKS_PER_UPDATE: u16 = 1;
    const DEFAULT_SYMBOL: &str = "ETHUSDT";

    pub fn with_symbol<S: Into<Symbol>>(symbol: S) -> Result<Self> {
        let symbol = symbol.into();
        let market = Arc::new(Market::new(None, None));
        let live_stats_tracker = LiveStatsTracker::new(market.clone(), symbol);
        Ok(Self {
            market,

            symbol,
            live_stats_tracker,

            price_triggers: vec![
                PriceTrigger {
                    price: PriceLevel(1210.0),
                    condition: TriggerCondition::HigherEq,
                },
                PriceTrigger {
                    price: PriceLevel(1208.0),
                    condition: TriggerCondition::LowerEq,
                },
                PriceTrigger {
                    price: PriceLevel(1209.0),
                    condition: TriggerCondition::HigherEq,
                },
            ],
            latest_alerts: Vec::new(),

            //tick: 0,
        })
    }

    pub fn new() -> Result<Self> {
        Self::with_symbol(Self::DEFAULT_SYMBOL)
    }

    pub fn analyze(&mut self) {
        let price = self.live_stats().last_price;

        let triggered = self
            .price_triggers
            .iter().enumerate()
            .filter(|(i, trigger)| match trigger.condition {
                TriggerCondition::HigherEq => price >= trigger.price.0,
                TriggerCondition::LowerEq => price <= trigger.price.0,
            })
            .collect::<Vec<(usize, &PriceTrigger)>>();

        if !triggered.is_empty() {
            for (i, t) in triggered {
                self.latest_alerts.push(Alert::new(t.price, "Price crossed over trigger zone!!!"));
            }
        }
    }

    /// Increments the inner ticker, updates the `live price stats` and schedules
    /// price analysis for each `TICKS_PER_UPDATE`.
    pub fn update(&mut self) {
        self.live_stats_tracker.update();

        //self.tick += 1;
        //if self.tick >= Self::TICKS_PER_UPDATE {
            self.analyze();
            //self.tick = 0;
        //}
    }

    pub fn alert(&self) -> Vec<Alert> {
        self.latest_alerts.clone()
    }

    // TODO maybe do inlining
    pub fn live_stats(&self) -> Arc<PriceStats> {
        self.live_stats_tracker.stats()
    }
}

#[derive(Debug)]
pub struct LiveStatsTracker {
    stats: Arc<PriceStats>,
    reader: Receiver<BinanceResult<PriceStats>>,
}

impl LiveStatsTracker {
    fn new(market: Arc<Market>, symbol: Symbol) -> Self {
        let reader = Self::spawn_price_reader(market, symbol);
        Self {
            stats: Arc::new(DEFAULT_PRICE_STATS),
            reader,
        }
    }

    fn update(&mut self) {
        if let Some(price) = self.reader.try_iter().last() {
            match price {
                Ok(stats) => self.stats = Arc::new(stats),
                Err(err) => println!("Binance Error: {err}"),
            }
        }
    }

    fn stats(&self) -> Arc<PriceStats> {
        self.stats.clone()
    }

    /// Reading the price from Binance charts blocks the thread for a short period of time
    /// which can sometimes delay the user input so a new thread is needed.
    ///
    /// Every [`crate::TICK_INTERVAL`] this thread reads the market price and sends it to
    /// the main thread which stores it in the next [`crate::TICK_INTERVAL`].
    ///
    /// If the price reader thread loses connection with the main thread it will just exit
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

impl From<&'static str> for Symbol {
    fn from(str: &'static str) -> Symbol {
        Symbol(str)
    }
}

impl From<Symbol> for String {
    fn from(symbol: Symbol) -> String {
        symbol.0.to_owned()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Alert {
    pub price: PriceLevel,
    pub message: &'static str,
}

impl Alert {
    fn new(price: PriceLevel, message: &'static str) -> Self {
        Self { price, message }
    }
}

#[derive(Debug)]
struct PriceTrigger {
    price: PriceLevel,
    condition: TriggerCondition,
}

#[derive(Debug)]
enum TriggerCondition {
    HigherEq,
    LowerEq,
}

/// Represents a single price level.
#[derive(Debug, Clone, Copy)]
pub struct PriceLevel(pub f64);

impl Display for PriceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Price: {}", self.0)
    }
}

pub const DEFAULT_PRICE_STATS: PriceStats = PriceStats {
    symbol: String::new(),
    price_change: String::new(),
    price_change_percent: String::new(),
    weighted_avg_price: String::new(),
    prev_close_price: 0.0,
    last_price: 0.0,
    bid_price: 0.0,
    ask_price: 0.0,
    open_price: 0.0,
    high_price: 0.0,
    low_price: 0.0,
    volume: 0.0,
    open_time: 0,
    close_time: 0,
    first_id: 0,
    last_id: 0,
    count: 0,
};
