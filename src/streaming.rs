use chrono::{ DateTime, Utc };
use futures::{ future, Stream };
use futures_util::{SinkExt, StreamExt };
use serde::{ Deserialize, Serialize };
use std::sync::{ mpsc, Arc, Mutex };
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::{ util, AccountStatus, Alpaca, Order };

#[derive(Debug, Deserialize, PartialEq)]
pub enum AuthorizationStatus {
   #[serde(rename="authorized")] Authorized,
   #[serde(rename="unauthorized")] Unauthorized
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum AuthorizationAction {
   #[serde(rename="authenticate")] Authenticate,
   #[serde(rename="listen")] Listen
}

#[derive(Debug, Deserialize)]
pub struct Authorization {
   status: AuthorizationStatus,
   action: AuthorizationAction
}

/// An update that has occurred due to an account change.
/// NOTE - this is not well documented in Alpaca and the fields might be different...
#[derive(Debug, Deserialize)]
pub struct AccountEvent {
   /// Account ID - a UUID
   pub id: String,

   /// Timestamp this account was created at
   #[serde(rename = "created_at")] pub created: DateTime<Utc>,

   /// Timestamp this account was updated at
   #[serde(rename = "updated_at")] pub updated: DateTime<Utc>,

   /// Timestamp this account was delete at
   #[serde(rename = "deleted_at")] pub deleted: Option<DateTime<Utc>>,

   /// Account status
   pub status: AccountStatus,

   /// Cash balance
   #[serde(deserialize_with = "util::to_f64")] pub cash: f64,

   /// ? The amount of cash that can be withdrawn ?
   #[serde(deserialize_with = "util::to_f64")] pub cash_withdrawable: f64
}


/// An event that has occured due to an order.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum OrderEvent {
   /// Sent when the order has been completed for the day - it is either “filled” or “done_for_day” - but
   /// remaining settlement calculations are still pending.
   Calculated { order: Order },

   /// Sent when your requested cancelation of an order is processed.
   Canceled { timestamp: DateTime<Utc>, order: Order },

   /// Sent when the order is done executing for the day, and will not receive further updates until the next trading day.
   DoneForDay { order: Order },

   /// Sent when an order has reached the end of its lifespan, as determined by the order’s time in force value.
   Expired { timestamp: DateTime<Utc>, order: Order },

   /// Sent when your order has been completely filled.
   Fill {
      timestamp: DateTime<Utc>,
      #[serde(deserialize_with = "util::to_f64")] price: f64,
      #[serde(deserialize_with = "util::to_u32", rename(deserialize="position_qty"))] qty: u32,
      order: Order
   },

   /// Sent when an order has been routed to exchanges for execution.
   New { order: Order },

   /// Sent when the order cancel has been rejected.
   OrderCancelRejected { order: Order },

   /// Sent when the order replace has been rejected.
   OrderReplaceRejected { order: Order },

   /// Sent when a number of shares less than the total remaining quantity on your order has been filled.
   PartialFill {
      timestamp: DateTime<Utc>,
      #[serde(deserialize_with = "util::to_f64")] price: f64,
      #[serde(deserialize_with = "util::to_u32", rename(deserialize="position_qty"))] qty: u32,
      order: Order
   },

   /// Sent when the order is awaiting cancelation. Most cancelations will occur without the order entering this state.
   PendingCancel { order: Order },

   /// Sent when the order has been received by Alpaca and routed to the exchanges, but has not yet been accepted for execution.
   PendingNew { order: Order },

   /// Sent when the order is awaiting replacement.
   PendingReplace { order: Order },

   /// Sent when your order has been rejected.
   Rejected { timestamp: DateTime<Utc>, order: Order },

   /// Sent when your requested replacement of an order is processed.
   Replaced { timestamp: DateTime<Utc>, order: Order },

   /// Sent when your order has been stopped, and a trade is guaranteed for the order, usually at a stated price or better,
   /// but has not yet occurred.
   Stopped { order: Order },

   /// Sent when the order has been suspended and is not eligible for trading.
   Suspended { order: Order },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListenStream {
   streams: Vec<String>
}

/// The possible actions we can push on to streams
#[derive(Debug, Serialize)]
#[serde(content = "data", rename_all="snake_case", tag = "action")]
enum ActionMessage {
   Listen(ListenStream),
}

/// The possible event streams that we can listen on
#[derive(Debug, Deserialize)]
#[serde(content = "data", tag = "stream")]
pub enum StreamMessage {
   /// This stream provides clients with updates pertaining to their brokerage accounts at Alpaca,
   /// including balance information
   #[serde(rename = "account_updates")] Account(AccountEvent),

   #[serde(rename = "authorization")] Authorization(Authorization),

   #[serde(rename = "listening")] Listening(ListenStream),

   /// This stream provides clients with updates pertaining to orders placed at Alpaca.  This includes
   /// order fills, partial fills, as well as cancellations and rejections of orders
   #[serde(rename = "trade_updates")] Order(OrderEvent),
}



/// Realtime event streamer
///
/// Currently streams updates to orders and the account.  To use the streamer, first create a new one
/// and then listen on the stream of events coming in.
///
/// # Example
///
/// To listen on the stream of events:
///
/// ``` no run
/// let alpaca = Alpaca::live("KEY_ID", "SECRET").await.unwrap();
///
/// let streamer = Streamer:new(&alpaca);
/// streamer.start().await
///    .for_each(|msg| {
///       match msg {
///          StreamMessage::Account(_) => println!("Got an account update!"),
///          StreamMessage::Order(_) => println!("Got an order update!"),
///          _ => println!("Got an unexpected msg")
///       }
///       future::ready(())
///    })
///    .await;
/// ```
pub struct Streamer<'a> {
   alpaca: &'a Alpaca,
   shutdown: Arc<Mutex<bool>>
}
impl<'a> Streamer<'a> {
   /// Creates a new event streamer.
   pub fn new(alpaca: &'a Alpaca) -> Streamer<'a> { Streamer { alpaca, shutdown: Arc::new(Mutex::new(false)) } }

   /// Starts the stream of events
   pub async fn start(&self) -> impl Stream<Item = StreamMessage> {
      let (host, auth_block) = self.alpaca.stream();
      let (tx, rx) = mpsc::channel();

      let (stream, _) = connect_async(host).await.unwrap();
      let (mut sink, source) = stream.split();

      // First - authenticate & set up the stream we want to listen on
      //         right now listen on all streams.  TODO - make it configurable
      let listen_msg = ActionMessage::Listen(ListenStream { streams: vec!["trade_updates".to_string(), "account_updates".to_string()] });
      let msg = serde_json::to_string(&listen_msg).unwrap();
      tx.send(Message::Text(auth_block)).unwrap();
      tx.send(Message::Text(msg)).unwrap();

      // spawn a separate thread for sending out messages
      let shutdown = self.shutdown.clone();
      tokio::spawn(async move {
         loop {
            // stop on shutdown notification
            if *(shutdown.lock().unwrap()) { break; }

            // we're still running - so get a message and send it out.
            // TODO - change this to WAIT on receive so that we don't block shutdown
            let msg = rx.recv().unwrap();
            sink.send(msg).await.unwrap();
         }
      });

      // Next - set up our stream & remap stuff coming in
      let pong_tx = tx.clone();
      let shutdown = self.shutdown.clone();
      source
         .filter_map(move |msg| {
            match msg.unwrap() {
               Message::Ping(_) => { pong_tx.send(Message::Pong("pong".as_bytes().to_vec())).unwrap(); },
               Message::Close(_) => { *(shutdown.lock().unwrap()) = true; },
               Message::Text(value) => { return future::ready(Some(value)); },
               Message::Binary(value) => { return future::ready(Some(String::from_utf8(value).unwrap())); },
               _ => {}
            };
            return future::ready(None)
         })
         .filter_map(|msg| {
            match serde_json::from_str(&msg).unwrap() {
               StreamMessage::Order(order) => future::ready(Some(StreamMessage::Order(order))),
               StreamMessage::Account(account) => future::ready(Some(StreamMessage::Account(account))),
               _ => future::ready(None)
            }
         })
   }

   /// Stops the stream of events
   pub fn stop(&mut self) {
      let mut shutdown = self.shutdown.lock().unwrap();
      *shutdown = true;
   }
}

#[cfg(test)]
mod test {
   use super::*;

   #[test]
   fn authenticate() {
      // GiVEN - a valid response block
      let response = r#"{"stream":"authorization","data":{"action":"authenticate","status":"authorized"}}"#;

      // WHEN - we parse it
      let x: StreamMessage = serde_json::from_str(response).unwrap();

      // THEN  - it's good
      match x {
         StreamMessage::Authorization(response) => {
            assert_eq!(AuthorizationAction::Authenticate, response.action);
            assert_eq!(AuthorizationStatus::Authorized, response.status);
         }
         _ => panic!("Wrong stream message")
      }
}
}