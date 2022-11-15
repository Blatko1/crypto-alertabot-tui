use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread,
};

use coingecko::CoinGeckoClient;
use tokio::{runtime::Runtime, task::JoinHandle};

use crate::error::Result;

pub struct Bot {
    client: Arc<CoinGeckoClient>,

    coin_symbol: Symbol,
    fiat_symbol: Symbol,
    price_tracker: PriceTracker,

    tick: u16,
}

// TODO replace unwrap() with ?
impl Bot {
    const TICKS_PER_UPDATE: u16 = 2;
    const DEFAULT_COIN: &str = "ETH";
    const DEFAULT_FIAT: &str = "USD";

    pub fn with_symbols<S: Into<Symbol>>(coin_symbol: S, fiat_symbol: S) -> Result<Self> {
        let coin_symbol = coin_symbol.into();
        let fiat_symbol = fiat_symbol.into();
        let client = Arc::new(CoinGeckoClient::default());
        let price_tracker = PriceTracker::new(client.clone(), coin_symbol, fiat_symbol)?;
        Ok(Self {
            client,

            coin_symbol,
            fiat_symbol,
            price_tracker,

            tick: 0,
        })
    }

    pub fn new() -> Result<Self> {
        Self::with_symbols(Self::DEFAULT_COIN, Self::DEFAULT_FIAT)
    }

    pub fn analyze(&mut self) {}

    pub fn tick(&mut self) -> Result<()> {
        self.price_tracker.track()?;
        self.tick += 1;

        if self.tick >= Self::TICKS_PER_UPDATE {
            self.analyze();
            self.tick = 0;
        }

        Ok(())
    }

    // TODO maybe do inlining
    pub fn get_price(&self) -> PriceLevel {
        self.price_tracker.get_price()
    }

    pub fn get_coin_symbol(&self) -> Symbol {
        self.coin_symbol
    }

    pub fn get_fiat_symbol(&self) -> Symbol {
        self.fiat_symbol
    }
}

struct PriceTracker {
    price: PriceLevel,
    runtime: Runtime,
    reader: Receiver<HashMap<String, coingecko::response::simple::Price>>,
}

impl PriceTracker {
    fn new(market: Arc<CoinGeckoClient>, coin_symbol: Symbol, fiat_symbol: Symbol) -> Result<Self> {
        let runtime = Runtime::new()?;
        let reader = Self::spawn_price_reader(&runtime, market, coin_symbol, fiat_symbol);
        Ok(Self {
            price: PriceLevel::NAN,
            runtime,
            reader
        })
    }

    fn track(&mut self) -> Result<()> {
        if let Some(price) = self.reader.try_iter().last() {
            //self.price = price.unwrap().into();
            println!("prie: {:?}", price.iter());
        }
        Ok(())
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
        runtime: &Runtime,
        client: Arc<CoinGeckoClient>,
        coin_symbol: Symbol,
        fiat_symbol: Symbol,
    ) -> 
        Receiver<HashMap<String, coingecko::response::simple::Price>> {
        let (tx, rx) = channel();
        runtime.spawn(async move {
            loop {
                let result = client
                    .price(&[coin_symbol.0], &[fiat_symbol.0], true, true, true, true)
                    .await;

                let price = match result {
                    Ok(price) => price,
                    Err(err) => { println!("CoinGecko Price Reader Error: {}", err); continue; },
                };
                let send = tx.send(price);
                match send {
                    Ok(_) => tokio::time::sleep(crate::TICK_INTERVAL).await,
                    Err(_) => break,
                };
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
