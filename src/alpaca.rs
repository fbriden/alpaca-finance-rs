use reqwest::{ Client, Method, RequestBuilder, Url };
use serde::Serialize;
use snafu::ResultExt;
use std::env;

use crate::{error, Result};

const LIVE_API: &'static str = "https://api.alpaca.markets";
const PAPER_API: &'static str = "https://paper-api.alpaca.markets";


#[derive(Debug, Serialize)]
struct Authenticate {
   key_id: String,
   secret_key: String
}

#[derive(Debug, Serialize)]
#[serde(content = "data", rename_all="snake_case", tag = "action")]
enum ActionMessage {
   Authenticate(Authenticate),
}

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

   /// Builds a websocket stream against the configured host
   /// Handles authentication; errors out if credentials are wrong
   pub(crate) fn stream(&self) -> (String, String) {
      // first - update the URL for websockets
      let mut ws_host = self.host.clone();
      ws_host.replace_range(..4, "ws");
      ws_host.push_str("/stream");

      let authenticate = ActionMessage::Authenticate(Authenticate { key_id: self.api_key.clone(), secret_key: self.api_secret.clone() });
      let message = serde_json::to_string(&authenticate).context(error::InternalJSON).unwrap();

      (ws_host, message)
   }

   /// Creates an object for interacting with the LIVE API
   ///
   /// # Example
   ///
   /// To get the alpaca context for the live account
   ///
   /// ``` no run
   /// let alpaca = Alpaca::live("KEY_ID", "SECRET").await.unwrap();
   /// ```
   pub async fn live(api_key_id: &str, api_secret_key: &str) -> Result<Alpaca> { Alpaca::build(true, api_key_id, api_secret_key).await }

   /// Creates an object for interacting with the PAPER API
   ///
   /// # Example
   ///
   /// To get the alpaca context for the paper account
   ///
   /// ``` no run
   /// let alpaca = Alpaca::paper("KEY_ID", "SECRET").await.unwrap();
   /// ```
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