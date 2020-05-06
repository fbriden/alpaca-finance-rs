use reqwest::Method;
use serde::Deserialize;
use snafu::{ ensure, ResultExt };

use crate::{ error, util, Alpaca, Result };

/// The status of the account
///
/// Most likely, the account status is ACTIVE unless there is any problem. The account status
/// may get in ACCOUNT_UPDATED when personal information is being updated from the dashboard,
/// in which case you may not be allowed trading for a short period of time until the change
/// is approved.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
   /// The account information is being updated.
   AccountUpdated,

   // The account is active for trading.
   Active,

   // The final account approval is pending.
   ApprovalPending,

   /// The account is onboarding.
   Onboarding,

   // The account application has been rejected.
   Rejected,

   // The account application submission failed for some reason.
   SubmissionFailed,

   // The account application has been submitted for review.
   Submitted
}

/// Important information related to an account.
///
/// Including account status, funds available for trade, funds available for withdrawal, and
/// various flags relevant to an account’s ability to trade.
///
/// An account maybe be blocked for just for trades (trades_blocked flag) or for both trades and
/// transfers (account_blocked flag) if Alpaca identifies the account to engaging in any suspicious
/// activity.
///
/// Also, in accordance with FINRA’s pattern day trading rule, an account may be flagged for
/// pattern day trading (pattern_day_trader flag), which would inhibit an account from placing any
/// further day-trades.
#[derive(Debug, Deserialize)]
pub struct Account {
   /// Account ID - a UUID
   pub id: String,

   /// Account number - a string different from the account ID
   #[serde(rename = "account_number")] pub number: String,

   /// Cash balance
   #[serde(deserialize_with = "util::to_f64")] pub cash: f64,

   /// The total equity in the account = cash + long_market_value + short_market_value
   #[serde(deserialize_with = "util::to_f64")] pub equity: f64,

   /// Real-time MtM value of all long positions held in the account
   #[serde(deserialize_with = "util::to_f64")] pub long_market_value: f64,

   /// Real-time MtM value of all short positions held in the account
   #[serde(deserialize_with = "util::to_f64")] pub short_market_value: f64,

   /// Current available $ buying power
   #[serde(deserialize_with = "util::to_f64")] pub buying_power: f64,

   /// If true, the account activity by user is prohibited.
   #[serde(rename = "account_blocked")] pub is_account_blocked:  bool,

   /// If true, the account has been flagged as a pattern day trader
   #[serde(rename = "pattern_day_trader")] pub is_pattern_day_trader: bool,

   /// If true, the account is not allowed to place orders due to customer request.
   #[serde(rename = "trade_suspended_by_user")] pub is_trade_suspended: bool,

   /// If true, the account is not allowed to place orders.
   #[serde(rename = "trading_blocked")] pub is_trading_blocked: bool,

   /// If true, the account is not allowed to request money transfers.
   #[serde(rename = "transfers_blocked")] pub is_transfers_blocked: bool,

   /// Account status
   pub status: AccountStatus,
}
impl Account {
   /// Gets the current account information
   pub async fn get(alpaca: &Alpaca) -> Result<Account> {
      let response = alpaca.request(Method::GET, "v2/account")?
         .send().await.context(error::RequestFailed)?;
      ensure!(response.status().is_success(), error::InvalidCredentials);

      Ok(response.json::<Account>().await.context(error::BadData)?)
   }
}