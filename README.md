# Alpaca Finance

An API for interacting with [Alpaca][alpaca].

[![Package][cratesio-image]][cratesio]
[![Documentation][docsrs-image]][docsrs]
[![Build Status][build-image]][build]

[alpaca]: https://alpaca.markets
[docsrs-image]: https://docs.rs/alpaca-finance/badge.svg
[docsrs]: https://docs.rs/alpaca-finance
[cratesio-image]: https://img.shields.io/crates/v/alpaca-finance.svg
[cratesio]: https://crates.io/crates/alpaca-finance
[build-image]: https://github.com/fbriden/alpaca-finance-rs/workflows/Build/badge.svg
[build]: https://github.com/fbriden/alpaca-finance-rs/actions

* Account information

```rust
use alpaca_finance::{ Account, Alpaca };

#[tokio::main]
async fn main() {
   // Get a connection to the live API
   let alpaca = Alpaca::live("My KEY ID", "My Secret Key").await.unwrap();
   let account = Account::get(&alpaca).await.unwrap();

   println!("I have ${:.2} in my account.", account.cash)
}
```

* To place an order

```rust
use alpaca_finance::{ Account, Alpaca };

#[tokio::main]
async fn main() {
   // Get a connection to the live API
   let alpaca = Alpaca::paper("My KEY ID", "My Secret Key").await.unwrap();
   let order = Order::buy("AAPL", 100, OrderType::Limit, TimeInForce::DAY)
      .limit_price(100.0)
      .place(sandbox).await.unwrap();
}
```

* Listening on account or order changes

```rust
use alpaca_finance::{ Alpaca, Streamer, StreamMessage };
use futures::{ future, StreamExt };

#[tokio::main]
async fn main() {
   // Get a connection to the live API
   let alpaca = Alpaca::paper("My KEY ID", "My Secret Key").await.unwrap();

   let streamer = Streamer:new(&alpaca);
   streamer.start().await
      .for_each(|msg| {
         match msg {
            StreamMessage::Account(_) => println!("Got an account update!"),
            StreamMessage::Order(_) => println!("Got an order update!"),
            _ => println!("Got an unexpected msg")
         }
         future::ready(())
      })
      .await;
}
```

### Usage

Add this to your `Cargo.toml`:

```toml
alpaca-finance = "0.2"
```
