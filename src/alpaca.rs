use reqwest::{ Client, Method, RequestBuilder, Url };
use snafu::ResultExt;
use std::env;

use crate::{error, Result};

const LIVE_API: &'static str = "https://api.alpaca.markets";
const PAPER_API: &'static str = "https://paper-api.alpaca.markets";

/// Alpaca contextual information that needs to be supplied to all calls.
pub struct Alpaca {
   api_key: String,
   api_secret: String,
   host: String
}
impl Alpaca {
   /// Builds an alpaca object for either live or paper (sandbox) access
   async fn build(live: bool, api_key_id: &str, api_secret_key: &str) -> Result<Alpaca> {
      let host = if live { LIVE_API } else { PAPER_API };
      let alpaca = Alpaca {
         api_key: api_key_id.to_string(),
         api_secret: api_secret_key.to_string(),
         host: env::var("TEST_URL").unwrap_or(host.to_string()) // default to a unit testing URL first
      };

      // perform quick test
      let response = alpaca.request(Method::GET, "v2/clock")?
         .send().await.context(error::RequestFailed)?;
      if response.status().is_success() { return Ok(alpaca) }

      let status = response.status().as_u16();
      match status {
         401 => error::InvalidCredentials.fail()?,
         403 => error::InvalidCredentials.fail()?,
         _ => error::CallFailed{ url: response.url().to_string(), status }.fail()?
      }
   }

   /// Creates an object for interacting with the LIVE API
   pub async fn live(api_key_id: &str, api_secret_key: &str) -> Result<Alpaca> { Alpaca::build(true, api_key_id, api_secret_key).await }

   /// Creates an object for interacting with the PAPER API
   pub async fn paper(api_key_id: &str, api_secret_key: &str) -> Result<Alpaca> { Alpaca::build(false, api_key_id, api_secret_key).await }

   /// Internal helper to build up a request to Alpaca with credentials set
   pub(crate) fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
      let url = Url::parse(&self.host).context(error::InternalURL { url: &self.host})?
         .join(path).context(error::InternalURL { url: path })?;

      let client = Client::new();
      Ok(client.request(method, url)
         .header("APCA-API-KEY-ID", self.api_key.clone())
         .header("APCA-API-SECRET-KEY", self.api_secret.clone()))
   }
}