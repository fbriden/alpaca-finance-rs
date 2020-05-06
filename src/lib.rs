//! # Alpaca Finance
//!
//! Alpaca.markets provides a great set of APIs for developing algorithmic-based
//! stock trading.
//!
//! **ALWAYS VERIFY WITH THE PAPER API BEFORE USING THE LIVE API**
//!
//! Currently `alpaca_finance` provides:
//! * Access and authentication against the paper trading and live trading APIs
//! * Account API to get important information about your account
//! * Orders API to place, replace, cancel and get open orders.
//! * Realtime streaming updates to orders and account changes
//!
//! ## Quick Examples
//!
//! To find out how much cash you have in your account:
//!
//! ``` no run
//! use alpaca_finance::{ Account, Alpaca };
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
//! use alpaca_finance::{ Account, Alpaca };
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
//!
//! To watch for changes to orders or the account:
//!
//! ``` no run
//! use alpaca_finance::{ Alpaca, Streamer, StreamMessage };
//! use futures::{ future, StreamExt };
//!
//! #[tokio::main]
//! async fn main() {
//!    // Get a connection to the live API
//!    let alpaca = Alpaca::paper("My KEY ID", "My Secret Key").await.unwrap();
//!
//!    let streamer = Streamer:new(&alpaca);
//!    streamer.start().await
//!       .for_each(|msg| {
//!          match msg {
//!             StreamMessage::Account(_) => println!("Got an account update!"),
//!             StreamMessage::Order(_) => println!("Got an order update!"),
//!             _ => println!("Got an unexpected msg")
//!          }
//!          future::ready(())
//!       })
//!       .await;
//! }
//! ```


mod account;
pub use account::{ Account, AccountStatus };

mod alpaca;
pub use alpaca::Alpaca;

mod error;
use snafu::Snafu;

/// An opaque error hit when calling Alpaca
#[derive(Debug, Snafu)]
pub struct Error(error::InnerError);

/// The result of an operation
pub type Result<T> = std::result::Result<T, Error>;

mod order;
pub use order::{ Order, OrderBuilder, OrderStatus, OrderType, OrderUpdater, TimeInForce };

mod streaming;
pub use streaming::{ OrderEvent, Streamer, StreamMessage };

mod util;