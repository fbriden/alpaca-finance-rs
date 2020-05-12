use alpaca_finance::{ Order, OrderEvent };
use handlebars::{ no_escape, Handlebars };
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;

fn build_event(test_name: &str) -> String {
   let mut reg = Handlebars::new();
   reg.register_escape_fn(no_escape);

   // Load the simulated Yahoo data we want to test against
   let mut template_file = File::open(format!("tests/streaming_data/{}.hbs", test_name)).unwrap();
   let mut template = String::new();
   template_file.read_to_string(&mut template).unwrap();

   let mut order_file = File::open("tests/streaming_data/order.json").unwrap();
   let mut order = String::new();
   order_file.read_to_string(&mut order).unwrap();

   reg.render_template(&template, &json!({ "order": order })).unwrap()
}

fn validate_order(order: Order) {
   assert_eq!(15, order.qty);
   assert_eq!("AAPL", order.symbol);
}

#[test]
fn event_fill() {
   //! Ensure that we can parse fill events successfully

   // GIVEN - valid data for the 'fill' event
   let data = build_event("fill");

   // WHEN - we deserialize it
   let event = serde_json::from_str::<OrderEvent>(&data).unwrap();

   // THEN - we get the data we expect
   match event {
      OrderEvent::Fill { order, price, qty, timestamp: _ } => {
         assert_eq!(100, qty);
         assert_eq!(179.08, price);
         validate_order(order);
      },
      _ => panic!("Expected a fill order event")
   }
}