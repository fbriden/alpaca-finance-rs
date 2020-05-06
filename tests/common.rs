use alpaca_finance::Alpaca;
use mockito::{ mock, Mock };
use std::env;

const KEY_ID: &'static str = "someKey";
const SECRET: &'static str = "someSecret";

pub async fn build_alpaca() -> Alpaca {
   // Tell the actual code to use a test URL rather than the live one
   env::set_var("TEST_URL", mockito::server_url());

   // Set up the auth check
   let _validate = build_mock("GET", "/v2/clock").create();
   
   // and build our Alpaca client
   Alpaca::live(KEY_ID, SECRET).await.unwrap()
}

pub fn build_mock(verb: &'static str, path: &'static str) -> Mock {
   mock(verb, path)
      .with_header("APCA-API-KEY-ID", "asdf")
      .with_header("APCA-API-SECRET-KEY", "asdf")
      .with_header("content-type", "application/json")
      .with_status(200)
}