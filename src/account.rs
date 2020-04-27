use reqwest::Method;
use serde::Deserialize;
use super::{ util, Alpaca, Error };

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
   AccountUpdated,
   Active,
   ApprovalPending,
   Onboarding,
   Rejected,
   SubmissionFailed,
   Submitted
}

#[derive(Debug, Deserialize)]
pub struct Account {
   /// Account ID - a UUID
   pub id: String,

   /// Account number - a string different from the account ID
   #[serde(rename = "account_number")]
   pub number: String,

   /// Cash balance
   #[serde(deserialize_with = "util::to_f64")]
   pub cash: f64,

   /// The total equity in the account = cash + long_market_value + short_market_value
   #[serde(deserialize_with = "util::to_f64")]
   pub equity: f64,

   #[serde(deserialize_with = "util::to_f64")]
   pub long_market_value: f64,

   #[serde(deserialize_with = "util::to_f64")]
   pub short_market_value: f64,

   #[serde(deserialize_with = "util::to_f64")]
   pub buying_power: f64,

   pub status: AccountStatus
}
impl Account {
   /// Gets the current account information
   pub async fn get(alpaca: &Alpaca) -> Result<Account, Error> {
      let response = alpaca.request(Method::GET, "v2/account")?.send().await?;
      match response.status().is_success() {
         true => Ok(response.json::<Account>().await?),
         false => Err(Error::InvalidCredentials)
      }
   }
}