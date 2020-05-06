//! # Alpaca Finance
//!
//! Alpaca.markets provides a great set of APIs for developing algorithmic-based
//! stock trading.
//!
//! Current `alpaca_finance` provides:
//! * Access and authentication against the paper trading and live trading APIs
//! * Account API to get important information about your account
//! * Orders API to place, replace, cancel and get open orders.
//!
//! ## Quick Examples
//!
//! To find out how much cash you have in your account:
//!
//! ``` no run
//! use alpaca_finance::{Account, Alpaca};
//!
//! #[tokio::main]
//! async fn main() {
//!    // Get a connection to the live API
//!    let alpaca = Alpaca::live("My KEY ID", "My Secret Key").await.unwrap();
//!    let account = Account::get(&alpaca).await.unwrap();
//!
//!    println!("I have ${:.2} in my account.", account.cash)
//! }
//! ```
//!
//! To buy 100 shares of AAPL, through the paper API, at a limit price of $100.0 before the end of today:
//!
//! ``` no run
//! use alpaca_finance::{Account, Alpaca};
//!
//! #[tokio::main]
//! async fn main() {
//!    // Get a connection to the live API
//!    let alpaca = Alpaca::paper("My KEY ID", "My Secret Key").await.unwrap();
//!    let order = Order::buy("AAPL", 100, OrderType::Limit, TimeInForce::DAY)
//!       .limit_price(100.0)
//!       .place(sandbox).await.unwrap();
//! }
//! ```


mod account;
pub use account::{ Account, AccountStatus };

mod alpaca;
pub use alpaca::Alpaca;

mod error;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub struct Error(error::InnerError);

pub type Result<T> = std::result::Result<T, Error>;

mod order;
pub use order::{ Order, OrderStatus, OrderType, TimeInForce };

mod util;