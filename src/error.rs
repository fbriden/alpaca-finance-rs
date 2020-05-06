use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum InnerError {
   #[snafu(display("Alpaca is unavailable right now"))]
   AlpacaDown,

   #[snafu(display("Alpaca returned invalid data - {}", source.to_string()))]
   BadData { source: reqwest::Error },

   #[snafu(display("Alpaca call failed. '{}' returned a {} result.", url, status))]
   CallFailed { url: String, status: u16 },

   #[snafu(display("An internal error occurred"))]
   InternalJSON { source: serde_json::Error },

   #[snafu(display("An internal error occurred - please report that '{}' cannot be parsed because {}", url, source.to_string()))]
   InternalURL { url: String, source: url::ParseError },

   #[snafu(display("The key ID or secret key were not accepted"))]
   InvalidCredentials,

   #[snafu(display("The order cannot be submitted due to lack of buying power"))]
   OrderForbidden,

   #[snafu(display("The order is invalid.  {}", reason))]
   OrderInvalid { reason: String },

   #[snafu(display("The order '{}' cannot be canceled", order_id))]
   OrderNotCancelable { order_id: String },

   #[snafu(display("The order '{}' was not found", order_id))]
   OrderNotFound { order_id: String },

   #[snafu(display("Alpaca call failed for unknown reason."))]
   RequestFailed { source: reqwest::Error },

   #[snafu(display("Alpaca websocket connection failed for unknown reason."))]
   StreamingFailed { source: tungstenite::Error },

   #[snafu(display("An unexpected error occurred"))]
   Unknown
}