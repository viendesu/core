//! Serializer adapter confining any serializer to the shapes `serde_json` writes.

use std::fmt::Display;

use serde::ser::{self, Error as _, Impossible, Serialize, Serializer};

/// Serializes the value as `serde_json` would shape it: unit structs as null,
/// bytes as arrays of integers, non-finite floats as null, 128-bit integers as
/// 64-bit ones and map keys as strings.
pub struct JsonModel<'a, T: ?Sized>(pub &'a T);

impl<T: Serialize + ?Sized> Serialize for JsonModel<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(Shaper(serializer))
    }
}

struct Shaper<S>(S);

impl<S: Serializer> Serializer for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    type SerializeSeq = Shaper<S::SerializeSeq>;
    type SerializeTuple = Shaper<S::SerializeTuple>;
    type SerializeTupleStruct = Shaper<S::SerializeTupleStruct>;
    type SerializeTupleVariant = Shaper<S::SerializeTupleVariant>;
    type SerializeMap = Shaper<S::SerializeMap>;
    type SerializeStruct = Shaper<S::SerializeStruct>;
    type SerializeStructVariant = Shaper<S::SerializeStructVariant>;

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }

    fn serialize_bool(self, v: bool) -> Result<S::Ok, S::Error> {
        self.0.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<S::Ok, S::Error> {
        self.0.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<S::Ok, S::Error> {
        self.0.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<S::Ok, S::Error> {
        self.0.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<S::Ok, S::Error> {
        self.0.serialize_i64(v)
    }

    fn serialize_i128(self, v: i128) -> Result<S::Ok, S::Error> {
        if let Ok(v) = u64::try_from(v) {
            self.0.serialize_u64(v)
        } else if let Ok(v) = i64::try_from(v) {
            self.0.serialize_i64(v)
        } else {
            Err(S::Error::custom("number out of range"))
        }
    }

    fn serialize_u8(self, v: u8) -> Result<S::Ok, S::Error> {
        self.0.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<S::Ok, S::Error> {
        self.0.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<S::Ok, S::Error> {
        self.0.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<S::Ok, S::Error> {
        self.0.serialize_u64(v)
    }

    fn serialize_u128(self, v: u128) -> Result<S::Ok, S::Error> {
        match u64::try_from(v) {
            Ok(v) => self.0.serialize_u64(v),
            Err(_) => Err(S::Error::custom("number out of range")),
        }
    }

    fn serialize_f32(self, v: f32) -> Result<S::Ok, S::Error> {
        if v.is_finite() {
            self.0.serialize_f32(v)
        } else {
            self.0.serialize_unit()
        }
    }

    fn serialize_f64(self, v: f64) -> Result<S::Ok, S::Error> {
        if v.is_finite() {
            self.0.serialize_f64(v)
        } else {
            self.0.serialize_unit()
        }
    }

    fn serialize_char(self, v: char) -> Result<S::Ok, S::Error> {
        self.0.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<S::Ok, S::Error> {
        self.0.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<S::Ok, S::Error> {
        self.0.collect_seq(v)
    }

    fn serialize_none(self) -> Result<S::Ok, S::Error> {
        self.0.serialize_none()
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<S::Ok, S::Error> {
        self.0.serialize_some(&JsonModel(value))
    }

    fn serialize_unit(self) -> Result<S::Ok, S::Error> {
        self.0.serialize_unit()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<S::Ok, S::Error> {
        self.0.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
    ) -> Result<S::Ok, S::Error> {
        self.0.serialize_unit_variant(name, index, variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<S::Ok, S::Error> {
        // Unwrapped here: some serializers give special names a non-JSON meaning.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<S::Ok, S::Error> {
        self.0
            .serialize_newtype_variant(name, index, variant, &JsonModel(value))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, S::Error> {
        self.0.serialize_seq(len).map(Shaper)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, S::Error> {
        self.0.serialize_tuple(len).map(Shaper)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, S::Error> {
        self.0.serialize_tuple_struct(name, len).map(Shaper)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, S::Error> {
        self.0
            .serialize_tuple_variant(name, index, variant, len)
            .map(Shaper)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, S::Error> {
        self.0.serialize_map(len).map(Shaper)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, S::Error> {
        self.0.serialize_struct(name, len).map(Shaper)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, S::Error> {
        self.0
            .serialize_struct_variant(name, index, variant, len)
            .map(Shaper)
    }

    fn collect_str<T: Display + ?Sized>(self, value: &T) -> Result<S::Ok, S::Error> {
        self.0.collect_str(value)
    }
}

impl<S: ser::SerializeSeq> ser::SerializeSeq for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_element(&JsonModel(value))
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeTuple> ser::SerializeTuple for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_element(&JsonModel(value))
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeTupleStruct> ser::SerializeTupleStruct for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_field(&JsonModel(value))
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeTupleVariant> ser::SerializeTupleVariant for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_field(&JsonModel(value))
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeMap> ser::SerializeMap for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), S::Error> {
        self.0.serialize_key(&MapKey(key))
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_value(&JsonModel(value))
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), S::Error>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        self.0.serialize_entry(&MapKey(key), &JsonModel(value))
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeStruct> ser::SerializeStruct for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), S::Error> {
        self.0.serialize_field(key, &JsonModel(value))
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), S::Error> {
        self.0.skip_field(key)
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeStructVariant> ser::SerializeStructVariant for Shaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), S::Error> {
        self.0.serialize_field(key, &JsonModel(value))
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), S::Error> {
        self.0.skip_field(key)
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

/// Writes a map key as the string `serde_json` would make of it.
struct MapKey<'a, T: ?Sized>(&'a T);

impl<T: Serialize + ?Sized> Serialize for MapKey<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(KeyShaper(serializer))
    }
}

struct KeyShaper<S>(S);

fn key_must_be_a_string<E: ser::Error>() -> E {
    E::custom("key must be a string")
}

fn finite_key<S, F>(serializer: S, v: F) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    F: Into<f64> + Serialize + Copy,
{
    if !v.into().is_finite() {
        return Err(S::Error::custom("float key must be finite"));
    }
    let key = serde_json::to_string(&v).map_err(S::Error::custom)?;
    serializer.serialize_str(&key)
}

impl<S: Serializer> Serializer for KeyShaper<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    type SerializeSeq = Impossible<S::Ok, S::Error>;
    type SerializeTuple = Impossible<S::Ok, S::Error>;
    type SerializeTupleStruct = Impossible<S::Ok, S::Error>;
    type SerializeTupleVariant = Impossible<S::Ok, S::Error>;
    type SerializeMap = Impossible<S::Ok, S::Error>;
    type SerializeStruct = Impossible<S::Ok, S::Error>;
    type SerializeStructVariant = Impossible<S::Ok, S::Error>;

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }

    fn serialize_bool(self, v: bool) -> Result<S::Ok, S::Error> {
        self.0.serialize_str(if v { "true" } else { "false" })
    }

    fn serialize_i8(self, v: i8) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_i16(self, v: i16) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_i32(self, v: i32) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_i64(self, v: i64) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_i128(self, v: i128) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_u8(self, v: u8) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_u16(self, v: u16) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_u32(self, v: u32) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_u64(self, v: u64) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_u128(self, v: u128) -> Result<S::Ok, S::Error> {
        self.0.collect_str(&v)
    }

    fn serialize_f32(self, v: f32) -> Result<S::Ok, S::Error> {
        finite_key(self.0, v)
    }

    fn serialize_f64(self, v: f64) -> Result<S::Ok, S::Error> {
        finite_key(self.0, v)
    }

    fn serialize_char(self, v: char) -> Result<S::Ok, S::Error> {
        self.0.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<S::Ok, S::Error> {
        self.0.serialize_str(v)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<S::Ok, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<S::Ok, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, _value: &T) -> Result<S::Ok, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<S::Ok, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<S::Ok, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<S::Ok, S::Error> {
        self.0.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<S::Ok, S::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<S::Ok, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, S::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, S::Error> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T: Display + ?Sized>(self, value: &T) -> Result<S::Ok, S::Error> {
        self.0.collect_str(value)
    }
}
