use crate::quote::{Quote, Symbol, Side};
use rand::{thread_rng, Rng};
use tokio::sync::mpsc;
use tokio::time::{self, Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver};
use std::collections::{HashMap, VecDeque};
use log::{warn, error}; 


pub struct MarketMaker {
    max_rate: u32,
    current_rate: u32,
    last_rate_increase: Instant,
    sender: mpsc::Sender<Quote>,
	id_to_quote: HashMap<u64, Quote>,
    symbol_side_to_quotes: HashMap<(Symbol, Side), VecDeque<Quote>>,
}

impl MarketMaker {
    pub async fn new(min_rate: u32, max_rate: u32) -> Result<(MarketMaker, Receiver<Quote>), &'static str> {
        if min_rate == 0 || max_rate == 0 || min_rate > max_rate {
            return Err("Invalid rate parameters");
        }

        let buffer_duration_secs = 10; // How many seconds of quotes the buffer should hold
        let buffer_size = max_rate as usize * buffer_duration_secs;
        let (sender, receiver) = mpsc::channel(buffer_size);

        Ok((
            MarketMaker {
                max_rate,
                current_rate: min_rate,
                last_rate_increase: Instant::now(),
                sender,
				id_to_quote: HashMap::new(),
                symbol_side_to_quotes: HashMap::new(),
            },
            receiver,
        ))
    }

    pub async fn run(&mut self) {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            self.adjust_rate();
            for _ in 0..self.current_rate {
                let quote = MarketMaker::generate_quote().await;
            // Clone the quote for the sender
            if let Err(_) = self.sender.send(quote.clone()).await {
                error!("Error sending quote");
                break;
            }
            
            // Now use the original quote
            self.insert_quote(quote).await;
        }
    }
}

    async fn generate_quote() -> Quote {
    let mut rng = thread_rng();
    let symbol = rng.gen::<Symbol>();
    let side = rng.gen::<Side>();

    let base_price: f64;
    let fluctuation_range = 0.05; // Default fluctuation range

    // Determine base price based on the symbol.
    // No need for initial assignment as it's set in all match arms.
    match symbol {
        Symbol::BTCUSD => {
            base_price = 50_000.0; // Set base price for BTCUSD
        },
        Symbol::ETHUSD => {
            base_price = 1_850.0; // Set base price for ETHUSD
        },
        Symbol::ETHBTC => {
            base_price = 0.035; // Set base price for ETHBTC
        },
    };
    // Calculate price fluctuation.
    let fluctuation = rng.gen_range(-fluctuation_range..fluctuation_range) * base_price;
    let price = base_price + fluctuation;

    // Check and adjust final price to ensure it is not negative.
    if price < 0.0 {
        panic!("Make sure base price is non-negative and fluctuation percentage isn't above 100%");
    }

    // Generate a random size for the quote.
    let size = rng.gen_range(0.1..10.0);
    let id = rng.gen::<u64>();

    // Get the current time and convert it to a UNIX timestamp.
    let now = SystemTime::now();
    let ts = now.duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as u64;

    // Return a new Quote struct with the generated values.
    Quote {
        ts,
        sym: symbol,
        side: side,
        price,
        size,
        id,
    }
}


    fn adjust_rate(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_rate_increase).as_secs() >= 60 {
            self.current_rate = std::cmp::min(self.current_rate * 2, self.max_rate);
            self.last_rate_increase = now;
        }
    }
	async fn insert_quote(&mut self, quote: Quote) {
    // Check if the quote ID already exists and replace/update if necessary
    if let Some(existing_quote) = self.id_to_quote.get_mut(&quote.id) {
        *existing_quote = quote.clone();
        warn!("Updated existing quote with ID {}", quote.id);
    } else {
        self.id_to_quote.insert(quote.id, quote.clone());
    }

    // Get the VecDeque for the specific symbol and side combination
    let queue = self.symbol_side_to_quotes.entry((quote.sym, quote.side)).or_insert_with(VecDeque::new);

    // If the VecDeque has reached maximum capacity, remove the oldest quote
    if queue.len() == 100 {
        if let Some(old_quote) = queue.pop_front() {
            self.id_to_quote.remove(&old_quote.id);
            warn!("Reached maximum capacity, removed oldest quote with ID {}", old_quote.id);
        }
    }

    // Insert the new quote
    queue.push_back(quote);
}
}

// test cases
#[cfg(test)]
mod tests {
    use super::*;
    use crate::quote::{Quote, Symbol, Side};

    // Helper function to create a quote
    fn create_quote(id: u64, sym: Symbol, side: Side, price: f64) -> Quote {
        Quote {
            ts: 0, // Timestamp is not relevant for this test
            sym,
            side,
            price,
            size: 1.0,
            id,
        }
    }

    #[tokio::test]
    async fn test_update_existing_quote() {
        let mut maker = MarketMaker::new(1, 10).await.unwrap().0;
        let original_quote = create_quote(1, Symbol::BTCUSD, Side::Ask, 100.0);
        let updated_quote = create_quote(1, Symbol::BTCUSD, Side::Ask, 200.0);

        maker.insert_quote(original_quote).await;
        maker.insert_quote(updated_quote).await;

        assert_eq!(maker.id_to_quote.get(&1).unwrap().price, 200.0);
    }

    #[tokio::test]
    async fn test_quote_limit_per_symbol_and_side() {
        let mut maker = MarketMaker::new(1, 10).await.unwrap().0;

        for i in 0..150 {
            let quote = create_quote(i, Symbol::BTCUSD, Side::Ask, 100.0 + i as f64);
            maker.insert_quote(quote).await;
        }

        let quotes = maker.symbol_side_to_quotes.get(&(Symbol::BTCUSD, Side::Ask)).unwrap();
        assert_eq!(quotes.len(), 100);
        assert!(maker.id_to_quote.get(&0).is_none()); // First inserted quote should be gone
        assert!(maker.id_to_quote.get(&50).is_some()); // Later inserted quote should exist
    }
}
