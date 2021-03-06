use std::convert::TryFrom;
use std::io::{self, Cursor, ErrorKind};

use svm_types::{Type, WasmType, WasmValue};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::svm_byte_array;

///
/// This file contains the implementation of encoding & decoding of a `Vec<WasmValue>` into `svm_byte_array`.
/// (and vice-versa).
///
/// This encoding (and decoding) functionality should be also implemented by any SVM clients (e.g: C, Go).
/// The design motivation is sticking with `svm_byte_array` as the mechanism for passing data between SVM client
/// to SVM (via the `SVM C-API`)
///
////
/// ### Encoding format:
///
/// * First byte is for the number of WASM values.
///
/// * Then each WASM value is encoded as:
///   - A WASM value type. It can be `I32` or `I64` (SVM doesn't support Floats).
///     The encodings of `WasmType` to `u8` and of `WasmValue` to `u64` sit under the `svm-codec` crate.
///   - A Big-Endian encoding of the WASM value. `I32` consumes 4 bytes and `I64` 8 bytes.
///
/// +----------------------------------------------------------------------+
/// | #values  | value #1  |  value #1 |        |  value #N   |  value #N  |
/// | (1 byte) |  type     |  (4 or 8  | . . .  |    type     |   (4 or 8  |
/// |          | (1 byte)  |   bytes)  |        |   (1 byte)  |   bytes)   |
/// +----------+--------------------------------+-------------+------------+
///

/// Allocates a raw buffer destined to hold exactly `nvalues` WASM values.
/// The buffer is initialized with zeros.
pub fn alloc_wasm_values(nvalues: usize) -> svm_byte_array {
    let cap = wasm_values_capacity(nvalues);
    let ty = Type::of::<&[WasmValue]>();

    svm_byte_array::new(cap, ty)
}

/// Converts `svm_byte_array` into `Vec<WasmerValue>`
///
/// ```
/// use std::io;
/// use std::convert::TryFrom;
///
/// use svm_types::{WasmValue, Type};
/// use svm_ffi::svm_byte_array;
///
/// let ty = Type::of::<Vec<WasmValue>>();
/// let values = vec![WasmValue::I32(5), WasmValue::I64(10)];
///
/// let bytes: svm_byte_array = (ty, (&values)).into();
/// let vec: Result<_, io::Error> = Vec::<WasmValue>::try_from(&bytes);
///
/// assert_eq!(vec.unwrap(), values);
/// ```
impl From<(Type, &[WasmValue])> for svm_byte_array {
    fn from((ty, values): (Type, &[WasmValue])) -> svm_byte_array {
        let nvalues = values.len();

        assert!(nvalues <= std::u8::MAX as usize);

        let cap = wasm_values_capacity(nvalues);
        let mut bytes = Vec::with_capacity(cap);

        bytes.write_u8(nvalues as u8).unwrap();

        for value in values.iter() {
            let ty: WasmType = value.ty();
            bytes.write_u8(ty.into()).unwrap();

            let value: u64 = value.into();

            match ty {
                WasmType::I32 => bytes.write_u32::<BigEndian>(value as u32).unwrap(),
                WasmType::I64 => bytes.write_u64::<BigEndian>(value).unwrap(),
            };
        }

        (ty, bytes).into()
    }
}

impl From<(Type, Vec<WasmValue>)> for svm_byte_array {
    #[inline]
    fn from((ty, values): (Type, Vec<WasmValue>)) -> svm_byte_array {
        (ty, (&values)).into()
    }
}

impl From<(Type, &Vec<WasmValue>)> for svm_byte_array {
    #[inline]
    fn from((ty, values): (Type, &Vec<WasmValue>)) -> svm_byte_array {
        (ty, &values[..]).into()
    }
}

impl TryFrom<&svm_byte_array> for Vec<WasmValue> {
    type Error = io::Error;

    fn try_from(bytes: &svm_byte_array) -> Result<Self, Self::Error> {
        let slice: &[u8] =
            unsafe { std::slice::from_raw_parts(bytes.bytes, bytes.length as usize) };

        let length = slice.len();
        if length == 0 {
            return Err(ErrorKind::InvalidInput.into());
        }

        let nvalues = slice[0];

        let mut values = Vec::with_capacity(nvalues as usize);
        let mut cursor = Cursor::new(&slice[1..]);

        for _ in 0..nvalues {
            let ty = cursor.read_u8()?;
            let ty = WasmType::try_from(ty);

            let value: u64 = match ty {
                Ok(WasmType::I32) => cursor.read_u32::<BigEndian>()? as u64,
                Ok(WasmType::I64) => cursor.read_u64::<BigEndian>()?,
                _ => return Err(ErrorKind::InvalidInput.into()),
            };

            let value: WasmValue = (ty.unwrap(), value).into();
            values.push(value);
        }

        Ok(values)
    }
}

fn wasm_values_capacity(nvalues: usize) -> usize {
    assert!(nvalues <= std::u8::MAX as usize);

    // since allocation is a relatively expansive task,
    // we'd better allocate in a single-shot the maximum volume we'll need:
    //
    // 1. A single byte for `#values`
    // 2. Nine bytes for each value.
    //    * Each value is prepended with a single byte denoting its type
    //    * Each WASM value can consume at most 8 bytes
    1 + nvalues * 9
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tracking;

    fn raw_type_id<T: 'static>() -> usize {
        let ty = std::any::TypeId::of::<T>();
        let name = std::any::type_name::<T>();

        let ty = Type::TypeId(ty, name);

        tracking::interned_type(ty)
    }

    #[test]
    fn empty_vec_values_to_svm_byte_array() {
        let vec = Vec::<WasmValue>::new();

        let bytes: svm_byte_array = (Type::of::<Vec<WasmValue>>(), vec).into();
        let slice: &[u8] = bytes.into();

        let nvalues = slice[0];
        assert_eq!(nvalues, 0);
    }

    #[test]
    fn empty_svm_byte_array_to_vec_values_errors() {
        let bytes = svm_byte_array::default();
        let res: Result<Vec<WasmValue>, io::Error> = Vec::try_from(&bytes);

        assert!(res.is_err());
    }

    #[test]
    fn svm_byte_array_to_vec_values_with_zero_items() {
        let raw = vec![0];

        let bytes = svm_byte_array {
            bytes: raw.as_ptr(),
            length: raw.len() as u32,
            capacity: raw.capacity() as u32,
            type_id: raw_type_id::<Vec<WasmValue>>(),
        };

        let res: Result<Vec<WasmValue>, io::Error> = Vec::try_from(&bytes);

        assert_eq!(res.unwrap(), vec![]);
    }

    #[test]
    fn svm_byte_array_to_vec_values_with_missing_type_byte_error() {
        let raw = vec![1];

        let bytes = svm_byte_array {
            bytes: raw.as_ptr(),
            length: raw.len() as u32,
            capacity: raw.capacity() as u32,
            type_id: raw_type_id::<Vec<WasmValue>>(),
        };

        let res: Result<Vec<WasmValue>, io::Error> = Vec::try_from(&bytes);
        assert!(res.is_err());
    }

    #[test]
    fn svm_byte_array_to_vec_values_with_missing_value_bytes_error() {
        let raw = vec![1, WasmType::I32.into(), 0x10, 0x20];

        let bytes = svm_byte_array {
            bytes: raw.as_ptr(),
            length: raw.len() as u32,
            capacity: raw.capacity() as u32,
            type_id: raw_type_id::<Vec<WasmValue>>(),
        };

        let res: Result<Vec<WasmValue>, io::Error> = Vec::try_from(&bytes);
        assert!(res.is_err());
    }
}
