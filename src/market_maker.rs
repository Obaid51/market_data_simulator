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

        let (sender, receiver) = mpsc::channel(32); // Channel buffer size

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

    let (base_price, fluctuation_range) = match symbol {
        Symbol::BTCUSD => (50_000.0, 0.05), // Base price of 35,000 with ±5% fluctuation
        Symbol::ETHUSD => (1_850.0, 0.05),  // Base price of 1,250 with ±5% fluctuation
        Symbol::ETHBTC => (0.035, 0.05),    // Base price of 0.035 with ±5% fluctuation
    };

    // Calculate fluctuation
    let fluctuation = rng.gen_range(-fluctuation_range..fluctuation_range) * base_price;
    let price = base_price + fluctuation;

    let size = rng.gen_range(0.1..10.0);
    let id = rng.gen::<u64>();

    let now = SystemTime::now();
    let ts = now.duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64;

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
