use serde::{Deserializer, Serializer};

use super::*;
/// the underlying identifier for all mio ring items including entities, operations and specters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RingId {
    /// the millisecond timestamp of the entity's creation
    epoch: u128,
    /// the unique ord of the entity
    pub(crate) ord: usize,
}

impl RingId {
    pub fn now(ord: usize) -> Self {
        let epoch = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u128;
        Self { epoch, ord }
    }
}

impl Serialize for RingId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let RingId { epoch, ord } = self;
        let stem = format!("{:x}-{}", epoch, ord);
        serializer.serialize_str(&stem)
    }
}

impl<'de> Deserialize<'de> for RingId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let stem = String::deserialize(deserializer)?;
        let (epoch, ord) = {
            let mut iter = stem.split('-');
            let epoch = iter
                .next()
                .map(|s| u128::from_str_radix(s, 16))
                .ok_or_else(|| serde::de::Error::custom(format!("invalid ring id stem: {}", stem)))?
                .map_err(|e| {
                    serde::de::Error::custom(format!("invalid ring id stem: {}; {}", stem, e))
                })?;
            let ord = iter
                .next()
                .map(|s| s.parse())
                .ok_or_else(|| serde::de::Error::custom(format!("invalid ring id stem: {}", stem)))?
                .map_err(|e| {
                    serde::de::Error::custom(format!("invalid ring id stem: {}; {}", stem, e))
                })?;
            (epoch, ord)
        };
        Ok(Self { epoch, ord })
    }
}

/// the identifier for all entities and specters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into)]
pub struct MioId(RingId);
impl MioId {
    pub fn stem(&self) -> String {
        let MioId(id) = self;
        if let Ok(serde_json::Value::String(id)) = serde_json::to_value(id) {
            id
        } else {
            unreachable!("failed to serialize MioId")
        }
    }
}

/// the identifier for all operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into)]
pub struct OpId(RingId);
