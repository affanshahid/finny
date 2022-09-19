use rusty_money::iso;
use serde::de::Visitor;
use serde::Deserialize;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Currency(pub &'static iso::Currency);

impl Deref for Currency {
    type Target = iso::Currency;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct CurrencyVisitor;

impl<'de> Visitor<'de> for CurrencyVisitor {
    type Value = Currency;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing an ISO8601 currency")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        iso::find(v)
            .map(|c| Currency(c))
            .ok_or(E::custom(format!("currency not recognized: {}", v)))
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CurrencyVisitor)
    }
}
