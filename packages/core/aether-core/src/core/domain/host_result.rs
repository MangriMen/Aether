use extism::ToBytes;
use extism_convert::Msgpack;
use serde::{Deserialize, Serialize};
use serr::{SerializedError, ToSerializedError};

#[derive(Serialize, Deserialize, ToBytes, Debug)]
#[encoding(Msgpack)]
pub enum HostResult<T> {
    Ok(T),
    Err(SerializedError),
}

impl<T> From<crate::Result<T>> for HostResult<T> {
    fn from(res: crate::Result<T>) -> Self {
        match res {
            Ok(v) => Self::Ok(v),
            Err(e) => Self::Err(e.raw.to_serialized()),
        }
    }
}
