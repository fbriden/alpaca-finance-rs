use serde::{ de, Deserialize, Deserializer, Serializer };
use serde_json::Value;
use std::fmt::Display;

pub fn to_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
   Ok(match Value::deserialize(deserializer)? {
       Value::String(s) => s.parse().map_err(de::Error::custom)?,
       Value::Number(num) => num.as_f64().ok_or(de::Error::custom("Invalid number"))? as f64,
       _ => return Err(de::Error::custom("wrong type"))
   })
}

pub fn to_optional_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error> {
   #[derive(Deserialize)]
   struct Wrapper(#[serde(deserialize_with = "to_f64")] f64);

   let v = Option::deserialize(deserializer)?;
   Ok(v.map(|Wrapper(a)| a))   
}

pub fn to_i32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
   Ok(match Value::deserialize(deserializer)? {
       Value::String(s) => s.parse().map_err(de::Error::custom)?,
       Value::Number(num) => num.as_i64().ok_or(de::Error::custom("Invalid number"))? as i32,
       _ => return Err(de::Error::custom("wrong type"))
   })
}

pub fn to_string<T: Display, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
   serializer.collect_str(value)
}

pub fn to_optional_string<T: Display, S: Serializer>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error> {
   match value {
      Some(x) => serializer.collect_str(x),
      None => serializer.serialize_none()
   }
}