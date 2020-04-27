
use super::Error;
use reqwest::{ Client, Method, RequestBuilder, Url };

const LIVE_API: &'static str = "https://api.alpaca.markets";
const PAPER_API: &'static str = "https://paper-api.alpaca.markets";

pub struct Alpaca {
   api_key: String,
   api_secret: String,
   host: &'static str
}
impl Alpaca {
   async fn build(live: bool, api_key_id: &str, api_secret_key: &str) -> Result<Alpaca, Error> {
      let host = if live { LIVE_API } else { PAPER_API };
      let alpaca = Alpaca { api_key: api_key_id.to_string(), api_secret: api_secret_key.to_string(), host: host };

      // perform quick test
      let response = alpaca.request(Method::GET, "v2/clock")?.send().await?;
      if response.status().is_client_error() { return Err(Error::InvalidCredentials) }
      else if response.status().is_server_error() { return Err(Error::Unavailable) }
      else { Ok(alpaca) }
   }

   pub async fn live(api_key_id: &str, api_secret_key: &str) -> Result<Alpaca, Error> { Alpaca::build(true, api_key_id, api_secret_key).await }
   pub async fn sandbox(api_key_id: &str, api_secret_key: &str) -> Result<Alpaca, Error> { Alpaca::build(false, api_key_id, api_secret_key).await }

   pub(crate) fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, Error> {
      let url = Url::parse(self.host).unwrap().join(path).unwrap();

      let client = Client::new();
      Ok(client.request(method, url)
         .header("APCA-API-KEY-ID", self.api_key.clone())
         .header("APCA-API-SECRET-KEY", self.api_secret.clone()))
   }
}