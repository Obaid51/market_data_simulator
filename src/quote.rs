use std::fmt;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Symbol {
    BTCUSD,
    ETHUSD,
    ETHBTC,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Side {
    Ask,
    Bid,
}

// Implementing random generation for Symbol and Side
impl Distribution<Symbol> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Symbol {
        match rng.gen_range(0..=2) {
            0 => Symbol::BTCUSD,
            1 => Symbol::ETHUSD,
            _ => Symbol::ETHBTC,
        }
    }
}

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen_range(0..=1) {
            0 => Side::Ask,
            _ => Side::Bid,
        }
    }
}
#[derive(Clone)]
pub struct Quote {
    pub ts: i64,       // Timestamp
    pub sym: Symbol,   // Symbol
    pub side: Side,    // Side
    pub price: f64,    // Price
    pub size: f64,     // Size
    pub id: u64,       // Identifier
}



// Implementing Display trait for Quote
impl fmt::Display for Quote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "{},{:?},{:?},{:.7},{:.7},{}", 
            self.ts, self.sym, self.side, self.price, self.size, self.id
        )
    }
}