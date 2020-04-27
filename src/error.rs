use std::{ error, fmt };

#[derive(Debug)]
pub enum Error {
   InvalidCredentials,
   InvalidOrder(String),
   OrderForbidden,
   OrderNotCancelable(String),
   OrderNotFound(String),
   Unavailable,
   Unknown
}
impl error::Error for Error {
   fn source(&self) -> Option<&(dyn error::Error + 'static)> {
      None
   }
}
impl fmt::Display for Error {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match *self {
         Error::InvalidCredentials => write!(f, "The key ID or secret key were not accepted"),
         Error::InvalidOrder(ref reason) => write!(f, "The order is invalid.  {}", reason),
         Error::OrderForbidden => write!(f, "The order cannot be submitted due to lack of buying power"),
         Error::OrderNotCancelable(ref id) => write!(f, "The order {} cannot be canceled", id),
         Error::OrderNotFound(ref id) => write!(f, "The order {} was not found", id),
         Error::Unavailable => write!(f, "Alpaca is unavailable right now"),
         Error::Unknown => write!(f, "An unexpected error occurred"),
      }
   }
}
impl From<reqwest::Error> for Error {
   fn from(_: reqwest::Error) -> Self { Error::Unknown }
}