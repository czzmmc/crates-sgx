#![doc(html_root_url = "https://docs.rs/prost/0.5.0")]
#![cfg_attr(all(feature = "mesalock_sgx",
                not(target_env = "sgx")), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#[cfg(all(feature = "mesalock_sgx", not(target_env = "sgx")))]
#[macro_use]
extern crate sgx_tstd as std;

mod error;
mod message;
mod types;

#[doc(hidden)]
pub mod encoding;

pub use crate::error::{DecodeError, EncodeError};
pub use crate::message::Message;

use bytes::{BufMut, IntoBuf};

use crate::encoding::{decode_varint, encode_varint, encoded_len_varint};

// See `encoding::DecodeContext` for more info.
// 100 is the default recursion limit in the C++ implementation.
#[cfg(not(feature = "no-recursion-limit"))]
const RECURSION_LIMIT: u32 = 100;

/// Encodes a length delimiter to the buffer.
///
/// See [Message.encode_length_delimited] for more info.
///
/// An error will be returned if the buffer does not have sufficient capacity to encode the
/// delimiter.
pub fn encode_length_delimiter<B>(length: usize, buf: &mut B) -> Result<(), EncodeError>
where
    B: BufMut,
{
    let length = length as u64;
    let required = encoded_len_varint(length);
    let remaining = buf.remaining_mut();
    if required > remaining {
        return Err(EncodeError::new(required, remaining));
    }
    encode_varint(length, buf);
    Ok(())
}

/// Returns the encoded length of a length delimiter.
///
/// Applications may use this method to ensure sufficient buffer capacity before calling
/// `encode_length_delimiter`. The returned size will be between 1 and 10, inclusive.
pub fn length_delimiter_len(length: usize) -> usize {
    encoded_len_varint(length as u64)
}

/// Decodes a length delimiter from the buffer.
///
/// This method allows the length delimiter to be decoded independently of the message, when the
/// message is encoded with [Message.encode_length_delimited].
///
/// An error may be returned in two cases:
///
///  * If the supplied buffer contains fewer than 10 bytes, then an error indicates that more
///    input is required to decode the full delimiter.
///  * If the supplied buffer contains more than 10 bytes, then the buffer contains an invalid
///    delimiter, and typically the buffer should be considered corrupt.
pub fn decode_length_delimiter<B>(buf: B) -> Result<usize, DecodeError>
where
    B: IntoBuf,
{
    let mut buf = buf.into_buf();
    let length = decode_varint(&mut buf)?;
    if length > usize::max_value() as u64 {
        return Err(DecodeError::new(
            "length delimiter exceeds maximum usize value",
        ));
    }
    Ok(length as usize)
}

// Re-export #[derive(Message, Enumeration, Oneof)].
// Based on serde's equivalent re-export [1], but enabled by default.
//
// [1]: https://github.com/serde-rs/serde/blob/v1.0.89/serde/src/lib.rs#L245-L256
#[cfg(feature = "prost-derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate prost_derive;
#[cfg(feature = "prost-derive")]
#[doc(hidden)]
pub use prost_derive::*;