use reqwest::Method;
use serde::{ Deserialize, Serialize };
use std::fmt;
use super::{ util, Alpaca, Error };

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
   Buy,
   Sell
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
   /// The order has been received by Alpaca, but hasn’t yet been routed to the execution venue.
   Accepted,

   /// The order has been received by exchanges, and is evaluated for pricing.
   AcceptedForBidding,

   /// The order has been completed for the day (either filled or done for day), but remaining settlement calculations are still pending
   Calculated,

   /// The order has been canceled and no further updates will occur for the order.  This can be either due to a cancel request
   /// by the user, or the order has been canceled by the exchanges due to its time-in-force.
   Canceled,

   /// The order is done executing for he day, and will not recieve further updates until the next trading day.
   DoneForDay,

   /// The order has expired, and no further updates will occur for the order.
   Expired,

   /// The order has been filled, and no further updates will occur for the order.
   Filled,

   /// The order has been received by Alpaca and routed to exchanges for execution.  This is the usual initial state of an order.
   New,

   /// The order has been parially filled.
   PartiallyFilled,

   /// The order is waiting to be cancelled.
   PendingCancel,

   /// The order has been received by Alpaca, and routed to the exchanges, but has not yet been accepted for execution.
   PendingNew,

   /// The order is waiting to be replaced by another order.  The order will reject cancel request while in this state.
   PendingReplace,

   /// The order has been rejected, and no further updates will occur for the order.
   Rejected,

   /// The order was replaced by another order, or was updated due to a market event such as corporate action.
   Replaced,

   /// The order has been stopped, and a trade is guaranteed for the order, usually at a stated price or better, but has not yet occurred
   Stopped,

   /// The order has been suspended, and is not eligible for trading.
   Suspended
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
   Limit,
   Market,
   Stop,
   StopLimit
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeInForce {
   CLS,

   /// A day order is eligible for execution only on the day it is live. 
   DAY,

   /// A Fill or Kill order is only executed if the entire order quantity can be filled, otherwise the order is canceled.
   FOK,
   GTC,
   IOC,
   OPG
}
impl fmt::Display for TimeInForce {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

#[derive(Debug, Deserialize)]
pub struct Order {
   /// Account ID - a UUID
   pub id: String,

   /// Asset class
   pub asset_class: String,

   pub client_order_id: String,

   pub extended_hours: bool,

   #[serde(deserialize_with = "util::to_i32")]
   pub filled_qty: i32,

   #[serde(deserialize_with = "util::to_optional_f64")]
   pub filled_avg_price: Option<f64>,

   #[serde(deserialize_with = "util::to_optional_f64")]
   pub limit_price: Option<f64>,

   #[serde(rename = "type")]
   pub order_type: OrderType,

   /// Ordered quantity
   #[serde(deserialize_with = "util::to_i32")]
   pub qty: i32,

   /// Buy or Sell ?
   pub side: OrderSide,

   /// The status of the order
   pub status: OrderStatus,

   #[serde(deserialize_with = "util::to_optional_f64")]
   pub stop_price: Option<f64>,

   /// Asset symbol
   pub symbol: String,

   pub time_in_force: TimeInForce,
}
impl Order {
   pub async fn cancel(&self, alpaca: &Alpaca) -> Result<(), Error> {
      let response = alpaca.request(Method::DELETE, format!("v2/orders/{}", self.id).as_str())?
         .send().await?;

      if response.status().is_success() { return Ok(()) }
      match response.status().as_u16() {
         404 => Err(Error::OrderNotFound(self.id.clone())),
         422 => Err(Error::OrderNotCancelable(self.id.clone())),
         _ => Err(Error::Unknown)
      }
   }

   pub fn update(&self) -> OrderUpdater {
      OrderUpdater { id: self.id.clone(), ..Default::default() }
   }

   pub async fn get_open(alpaca: &Alpaca) -> Result<Vec<Order>, Error> {
      let response = alpaca.request(Method::GET, "v2/orders")?
         .query(&[("status", "open")])
         .send().await?;

      match response.status().is_success() {
         true => Ok(response.json::<Vec<Order>>().await?),
         false => Err(Error::InvalidCredentials)
      }
   }

   pub fn buy(symbol: &str, qty: i32, order_type: OrderType, time_in_force: TimeInForce) -> OrderBuilder {
      OrderBuilder { symbol: symbol.to_string(), qty: qty, side: OrderSide::Buy, order_type: order_type, time_in_force: time_in_force, ..Default::default() }
   }

   pub fn sell(symbol: &str, qty: i32, order_type: OrderType, time_in_force: TimeInForce) -> OrderBuilder {
      OrderBuilder { symbol: symbol.to_string(), qty: qty, side: OrderSide::Sell, order_type: order_type, time_in_force: time_in_force, ..Default::default() }
   }
}

#[derive(Debug, Serialize)]
pub struct OrderBuilder {
   extended_hours: bool,

   #[serde(skip_serializing_if = "Option::is_none")]
   limit_price: Option<f64>,

   #[serde(rename(serialize="type"))]
   order_type: OrderType,

   #[serde(serialize_with = "util::to_string")]
   qty: i32,

   side: OrderSide,

   #[serde(skip_serializing_if = "Option::is_none")]
   stop_price: Option<f64>,

   symbol: String,

   time_in_force: TimeInForce,
}
impl OrderBuilder {
   pub fn extended_hours(mut self, extended_hours: bool) -> OrderBuilder {
      self.extended_hours = extended_hours;
      self
   }

   pub fn limit_price(mut self, limit_price: f64) -> OrderBuilder {
      self.limit_price = Some(limit_price);
      self
   }

   pub fn stop_price(mut self, stop_price: f64) -> OrderBuilder {
      self.stop_price = Some(stop_price);
      self
   }

   pub async fn place(&self, alpaca: &Alpaca) -> Result<Order, Error> {
      if (self.order_type == OrderType::Limit || self.order_type == OrderType::StopLimit) && self.limit_price == None {
         return Err(Error::InvalidOrder("Limit orders need a limit price.".to_string()))
      }
      if (self.order_type == OrderType::Stop || self.order_type == OrderType::StopLimit) && self.stop_price == None {
         return Err(Error::InvalidOrder("Stop orders need a stop price.".to_string()))
      }
      if self.extended_hours && (self.order_type != OrderType::Limit || self.time_in_force != TimeInForce::DAY) {
         return Err(Error::InvalidOrder("Extended hours only works limit orders for today".to_string()))
      }

      let response = alpaca.request(Method::POST, "v2/orders")?
         .json::<OrderBuilder>(self)
         .send()
         .await?;

      if response.status().is_success() { return Ok(response.json::<Order>().await?) }

      match response.status().as_u16() {
         403 => Err(Error::OrderForbidden),
         _ => Err(Error::Unknown)
      }
   }
}
impl Default for OrderBuilder {
   fn default() -> Self {
      OrderBuilder {
         symbol: "".to_string(),
         extended_hours: false,
         qty: 0,
         side: OrderSide::Buy,
         order_type: OrderType::Market,
         time_in_force: TimeInForce::DAY,
         limit_price: None,
         stop_price: None
      }
   }
}

#[derive(Debug, Default, Serialize)]
pub struct OrderUpdater {
   #[serde(skip_serializing)]
   id: String,

   #[serde(skip_serializing_if = "Option::is_none")]
   #[serde(serialize_with = "util::to_optional_string")]
   limit_price: Option<f64>,

   #[serde(skip_serializing_if = "Option::is_none")]
   #[serde(serialize_with = "util::to_optional_string")]
   qty: Option<i32>,

   #[serde(skip_serializing_if = "Option::is_none")]
   #[serde(serialize_with = "util::to_optional_string")]
   stop_price: Option<f64>,

   #[serde(skip_serializing_if = "Option::is_none")]
   #[serde(serialize_with = "util::to_optional_string")]
   time_in_force: Option<TimeInForce>,
}
impl OrderUpdater {
   pub fn limit_price(mut self, limit_price: f64) -> OrderUpdater {
      self.limit_price = Some(limit_price);
      self
   }

   pub fn qty(mut self, qty: i32) -> OrderUpdater {
      self.qty = Some(qty);
      self
   }

   pub fn stop_price(mut self, stop_price: f64) -> OrderUpdater {
      self.stop_price = Some(stop_price);
      self
   }

   pub fn time_in_force(mut self, time_in_force: TimeInForce) -> OrderUpdater {
      self.time_in_force = Some(time_in_force);
      self
   }

   pub async fn place(&self, alpaca: &Alpaca) -> Result<Order, Error> {
      let x = serde_json::to_string(self).unwrap();
      println!("asdf {} - {}", x, self.id);

      let response = alpaca.request(Method::PATCH, format!("v2/orders/{}", self.id).as_str())?
         .json::<OrderUpdater>(self)
         .send()
         .await?;

      if response.status().is_success() { return Ok(response.json::<Order>().await?) }

      match response.status().as_u16() {
         403 => Err(Error::OrderForbidden),
         _ => Err(Error::Unknown)
      }
   }   
}