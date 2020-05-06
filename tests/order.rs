use alpaca_finance::{ Order };
use mockito::Mock;
use std::fs::File;
use std::io::prelude::*;
use tokio_test::block_on;

mod common;

async fn base_mock(test_name: &str, mock: Mock) -> std::io::Result<Mock> {
   // Load the simulated Yahoo data we want to test against
   let mut file = File::open(format!("tests/order_data/{}.json", test_name))?;
   let mut contents = String::new();
   file.read_to_string(&mut contents)?;

   Ok(mock.with_header("content-type", "application/json")
      .with_body(&contents)
      .with_status(200))
}

#[test]
fn get_open() {
   //! Ensure that we can load valid open orders

   // GIVEN - a valid open order in place
   let alpaca = block_on(common::build_alpaca());
   let _m = block_on(base_mock("valid_open", common::build_mock("GET", "/v2/orders?status=open"))).unwrap().create();

   // WHEN - we get our open orders
   let orders = block_on(Order::get_open(&alpaca)).unwrap();

   // THEN - we get the results we expect
   assert_eq!(1, orders.len());
   assert_eq!(orders[0].id, "904837e3-3b76-47ec-b432-046db621571b");
   assert_eq!(orders[0].client_order_id, "904837e3-3b76-47ec-b432-046db621571b");
}