mod market_maker;
mod quote;

use log::{info, error};
use log4rs;
use market_maker::MarketMaker;
use tokio::sync::mpsc;
use crate::quote::Quote;

#[tokio::main]
async fn main() {
    // Initialize the logger to log to a file
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap_or_else(|e| {
        eprintln!("Error initializing log4rs: {}", e);
        std::process::exit(1);
    });

    info!("MarketMaker application starting");

    let min_rate = 10000;
    let max_rate = 10000;

    // Initialize the MarketMaker
    let (mut maker, receiver) = match MarketMaker::new(min_rate, max_rate).await {
        Ok((maker, receiver)) => {
            info!("MarketMaker initialized successfully");
            (maker, receiver)
        },
        Err(e) => {
            error!("Error initializing Market Maker: {}", e);
            return;
        },
    };

    // Spawn an asynchronous task to handle incoming quotes
    tokio::spawn(async move {
        handle_quotes(receiver).await;
    });

    // Start the Market Maker
    maker.run().await;
}

// Asynchronous function to handle incoming quotes
async fn handle_quotes(mut receiver: mpsc::Receiver<Quote>) {
    while let Some(quote) = receiver.recv().await {
        println!("Received Quote: {}", quote);
    }
}
