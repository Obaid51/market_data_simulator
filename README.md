
# Synthetic Market Data Generator

## Overview

This project is a synthetic market data generator specifically designed for cryptocurrencies. It simulates real-world cryptocurrency market data, providing a valuable tool for testing and analysis in crypto-related software development.

## Prerequisites

Before building and running this project, ensure you have the following installed:

-   Rust Programming Language: https://www.rust-lang.org/tools/install
-   Cargo (Rust's package manager, usually installed with Rust)

## Building the Project

1.  **Clone the Repository:**
    

    
    `git clone https://github.com/Obaid51/market_data_simulator.git` 
    
2.  **Build the Project:**
    
    Use Cargo to build the project:
    
    `cargo build --release` 
    
    This command compiles the project and generates an executable in the `target/release` directory.
3.  **Run the Tests:**
    
    Use Cargo to run the tests:
    
    `cargo test --release` 
    
    This command runs the tests for the simulator.
    

## Running the Application

After building the project, you can run it using Cargo:

Copy code

`cargo run --release` 

Alternatively, you can directly execute the binary in the `target/release` directory.

## Configuration

-   **Logging:** Modify the `log4rs.yaml` file to configure logging settings. By default, informational logs are printed to the console, and error logs are written to a file in the `log` directory.
    
-   **Market Maker Settings:** Adjust the settings for the market maker in the `main.rs` to set the min and max rate of quotes.


## Contributing

Contributions to this project are welcome. Please follow these steps to contribute:

1.  Fork the repository.
2.  Create a new branch for your feature or bug fix.
3.  Implement your changes.
4.  Write or update tests as necessary.
5.  Push your branch and open a pull request.