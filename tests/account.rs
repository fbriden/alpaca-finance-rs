use alpaca_finance::{ Account, AccountStatus };
use mockito::Mock;
use std::fs::File;
use std::io::prelude::*;
use tokio_test::block_on;

mod common;

async fn base_mock(test_name: &str) -> std::io::Result<Mock> {
   // Load the simulated Yahoo data we want to test against
   let mut file = File::open(format!("tests/account_data/{}.json", test_name))?;
   let mut contents = String::new();
   file.read_to_string(&mut contents)?;

   Ok(common::build_mock("GET", "/v2/account")
      .with_header("content-type", "application/json")
      .with_body(&contents)
      .with_status(200))
}

#[test]
fn get_account() {
   //! Ensure that we can load a valid account

   // GIVEN - a valid account on Alpaca
   let alpaca = block_on(common::build_alpaca());
   let _m = block_on(base_mock("valid")).unwrap().create();

   // WHEN - we get our account
   let account = block_on(Account::get(&alpaca)).unwrap();

   // THEN - we get the results we expect
   assert_eq!("e6fe16f3-64a4-4921-8928-cadf02f92f98", account.id);
   assert_eq!("010203ABCD", account.number);
   assert_eq!(-23140.2, account.cash);
   assert_eq!(103820.56, account.equity);
   assert_eq!(126960.76, account.long_market_value);
   assert_eq!(0.0, account.short_market_value);
   assert_eq!(262113.632, account.buying_power);
   assert_eq!(AccountStatus::Active, account.status);
   assert_eq!(false, account.is_account_blocked);
   assert_eq!(false, account.is_trade_suspended);
   assert_eq!(false, account.is_trading_blocked);
   assert_eq!(false, account.is_transfers_blocked);
}

#[test]
#[should_panic(expected = "InvalidCredentials")]
fn get_account_bad_credentials() {
   //! Ensure that we fail gracefully when we have bad credentials

   // GIVEN - invalid credentials for an account
   let alpaca = block_on(common::build_alpaca());
   let _m = block_on(base_mock("valid")).unwrap()
      .with_status(403)
      .create();

   // WHEN - we get our account
   block_on(Account::get(&alpaca)).unwrap();

   // THEN - we get an error
}

#[test]
#[should_panic(expected = "BadData")]
fn get_account_bad_data() {
   //! Ensure that we fail gracefully when Alpaca sends us bad data

   // GIVEN - an account with corrupted data
   let alpaca = block_on(common::build_alpaca());
   let _m = block_on(base_mock("corrupted")).unwrap().create();

   // WHEN - we get our account
   block_on(Account::get(&alpaca)).unwrap();

   // THEN - we get an error
}