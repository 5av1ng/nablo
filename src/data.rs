//! data processing functions for `nablo`

use serde::de::*;
use serde::Deserializer;
use std::fmt::Display;
use serde::Serialize;
use serde::ser;

/// a enum that represent a value. tuple, array, struct will be parse as Node.
#[derive(PartialEq, Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub enum DataEnum {
	Node(Vec<ParsedData>),
	/// contains key and value
	Map(Box<(ParsedData, ParsedData)>),
	/// string contains enum value
	Enum(String, Vec<ParsedData>),
	Data(Vec<u8>),
	String(String),
	Int(i128),
	Float(f64),
	Bool(bool),
	#[default] None,
}

/// a struct that represent a struct, see more in [`DataEnum`]. Note: if a map's key is not one of string int float or bool, nablo will not deliver name field
#[derive(PartialEq, Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ParsedData {
	/// the value
	pub data: DataEnum,
	/// name of the value
	pub name: String,
	#[serde(skip)]
	need_delete: bool
}

/// all possible errors when parsing data.
#[derive(thiserror::Error, Debug)]
pub enum Error {
	/// error during serializing or deserializing
	#[error("error during serializing or deserializing... info: {0}")]
	SerdeError(String),
	/// unexpected type, contains expected type name
	#[error("error while deserializing elements, info: unexpected type, expect: {0}")]
	UnexpectedType(String),
	#[error("syntax error")]
	Syntax
}

impl serde::ser::Error for Error {
	fn custom<T: Display>(input: T) -> Self {
		Self::SerdeError(input.to_string())
	}
}

impl serde::de::Error for Error {
	fn custom<T: Display>(input: T) -> Self {
		Self::SerdeError(input.to_string())
	}
}

struct Parser {}

struct DeParser<'a> {
	data: &'a mut ParsedData
}

#[derive(Default)]
struct Layer {
	inner: Vec<ParsedData>,
	final_name: String,
}

struct DeLayer<'a> {
	inner: DeParser<'a>,
}

struct DeMap<'a> {
	inner: DeParser<'a>,
	temp: Option<ParsedData>
}

struct DeEnum<'a> {
	inner: &'a mut DeParser<'a>,
}

impl<'a> DeMap<'a> {
	fn from(inner: &'a mut ParsedData) -> Self {
		Self {
			inner: DeParser { data: inner },
			temp: None
		}
	}
}

impl<'a> DeLayer<'a> {
	fn from(inner: &'a mut ParsedData) -> Self {
		Self {
			inner: DeParser { data: inner }
		}
	}
}

impl Layer {
	fn new(final_name: impl Into<String>) -> Self {
		Self {
			final_name: final_name.into(),
			..Default::default()
		}
	}
}

/// parse a data into [`ParsedData`]
pub fn to_data<T: serde::Serialize>(input: &T) -> Result<ParsedData, Error> {
	let mut serializer = Parser {};
	input.serialize(&mut serializer)
}

/// parse a [`ParsedData`] data into your type
pub fn from_data<'a, T>(input: &mut ParsedData) -> Result<T, Error>
where
	T: serde::Deserialize<'a>
{
	let mut deserializer = DeParser {
		data: input
	};
	T::deserialize(&mut deserializer)
}


macro_rules! impl_into_parsed_data {
	($t: ty, $s: tt) => {
		impl From<$t> for ParsedData {
			fn from(input: $t) -> Self {
				ParsedData {
					data: DataEnum::$s(input.into()),
					name: "".to_string(),
					need_delete: false
				}
			}
		}
	};
}

macro_rules! impl_serdelize {
	($i: ident, $t: ty) => {
		fn $i(self, input: $t) -> Result<ParsedData, Error> {
			Ok(input.into())
		}
	};
}

impl_into_parsed_data!(bool, Bool);
impl_into_parsed_data!(i8, Int);
impl_into_parsed_data!(i16, Int);
impl_into_parsed_data!(i32, Int);
impl_into_parsed_data!(i64, Int);
impl_into_parsed_data!(u8, Int);
impl_into_parsed_data!(u16, Int);
impl_into_parsed_data!(u32, Int);
impl_into_parsed_data!(u64, Int);
impl_into_parsed_data!(f32, Float);
impl_into_parsed_data!(f64, Float);
impl_into_parsed_data!(char, String);
impl_into_parsed_data!(&str, String);
impl_into_parsed_data!(String, String);
impl_into_parsed_data!(&[u8], Data);
impl_into_parsed_data!(Vec<ParsedData>, Node);

impl<'a> ser::Serializer for &'a mut Parser {
	type Ok = ParsedData;
	type Error = Error;
	type SerializeSeq = Layer;
	type SerializeTuple = Layer;
	type SerializeTupleStruct = Layer;
	type SerializeTupleVariant = Layer;
	type SerializeMap = Layer;
	type SerializeStruct = Layer;
	type SerializeStructVariant = Layer;

	impl_serdelize!(serialize_bool, bool);
	impl_serdelize!(serialize_i8, i8);
	impl_serdelize!(serialize_i16, i16);
	impl_serdelize!(serialize_i32, i32);
	impl_serdelize!(serialize_i64, i64);
	impl_serdelize!(serialize_u8, u8);
	impl_serdelize!(serialize_u16, u16);
	impl_serdelize!(serialize_u32, u32);
	impl_serdelize!(serialize_u64, u64);
	impl_serdelize!(serialize_f32, f32);
	impl_serdelize!(serialize_f64, f64);
	impl_serdelize!(serialize_char, char);
	impl_serdelize!(serialize_str, &str);
	impl_serdelize!(serialize_bytes, &[u8]);

	fn serialize_none(self) -> Result<ParsedData, Error> {
		Ok(ParsedData {
			data: DataEnum::None,
			name: "".to_string(),
			need_delete: false
		})
	}

	fn serialize_some<T: ?Sized + Serialize>(self, input: &T) -> Result<ParsedData, Error> {
		input.serialize(self)
	}

	fn serialize_unit(self) -> Result<ParsedData, Error> {
		self.serialize_none()
	}

	fn serialize_unit_struct(self, _: &'static str) -> Result<ParsedData, Error> {
		Ok(ParsedData {
			data: DataEnum::None,
			name: "".to_string(),
			need_delete: false
		})
	}

	fn serialize_unit_variant(self, _: &'static str, _: u32, input: &'static str) -> Result<ParsedData, Error> {
		Ok(ParsedData {
			data: DataEnum::Enum(input.into(), vec!()),
			name: "".to_string(),
			need_delete: false
		})
	}

	fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _: &'static str, _: u32, variant: &'static str, value: &T) -> Result<ParsedData, Error> {
		let back = value.serialize(self)?;
		Ok(ParsedData{
			data: DataEnum::Enum(variant.into(), vec!(back)),
			name: "".to_string(),
			need_delete: false
		})
	}

	fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, value: &T) -> Result<ParsedData, Error> {
		value.serialize(self)
	}
	fn serialize_seq(self, _: Option<usize>) -> Result<Layer, Error> { Ok(Layer::default()) }
	fn serialize_tuple(self, _: usize) -> Result<Layer, Error> { Ok(Layer::default()) }
	fn serialize_struct(self, name: &'static str, _: usize ) -> Result<Layer, Error> { Ok(Layer::new(name)) }
	fn serialize_tuple_struct(self, name: &'static str, _: usize) -> Result<Layer, Error> { Ok(Layer::new(name)) }
	fn serialize_tuple_variant(self, _: &'static str, _: u32, variant: &'static str, _: usize) -> Result<Layer, Error> { Ok(Layer::new(variant)) }
	fn serialize_map(self, _:  Option<usize>) -> Result<Layer, Error> { Ok(Layer::default()) }
	fn serialize_struct_variant(self, _: &'static str, _: u32, variant: &'static str, _: usize) -> Result<Layer, Error> { Ok(Layer::new(variant)) }
}

impl ser::SerializeSeq for Layer {
	type Ok = ParsedData;
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T)  -> Result<(), Error> 
	where
		T: ?Sized + Serialize,
	{
		self.inner.push(value.serialize(&mut Parser {})?);
		Ok(())
	}
	fn end(self) -> Result<ParsedData, Error> { 
		Ok(ParsedData {
			name: self.final_name.clone(), 
			..self.inner.into()
		}) 
	}
}

impl ser::SerializeTuple for Layer {
	type Ok = ParsedData;
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T)  -> Result<(), Error> 
	where
		T: ?Sized + Serialize,
	{
		self.inner.push(value.serialize(&mut Parser {})?);
		Ok(())
	}
	fn end(self) -> Result<ParsedData, Error> { 
		Ok(ParsedData {
			name: self.final_name.clone(), 
			..self.inner.into()
		}) 
	}
}

impl ser::SerializeTupleStruct for Layer {
	type Ok = ParsedData;
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T)  -> Result<(), Error> 
	where
		T: ?Sized + Serialize,
	{
		self.inner.push(value.serialize(&mut Parser {})?);
		Ok(())
	}
	fn end(self) -> Result<ParsedData, Error> { 
		Ok(ParsedData {
			name: self.final_name.clone(), 
			..self.inner.into()
		}) 
	}
}

impl ser::SerializeTupleVariant for Layer {
	type Ok = ParsedData;
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T)  -> Result<(), Error> 
	where
		T: ?Sized + Serialize,
	{
		self.inner.push(value.serialize(&mut Parser {})?);
		Ok(())
	}

	fn end(self) -> Result<ParsedData, Error> { 
		Ok(ParsedData {
			name: self.final_name.clone(), 
			data: DataEnum::Enum(self.final_name.clone(), self.inner.into()),
			need_delete: false
		}) 
	}
}

impl ser::SerializeMap for Layer {
	type Ok = ParsedData;
	type Error = Error;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
	where
		T: ?Sized + Serialize,
	{
		let key = key.serialize(&mut Parser {})?;
		let name = match key.data {
			DataEnum::String(ref inner) => inner.to_string(),
			DataEnum::Int(inner) => inner.to_string(),
			DataEnum::Float(inner) => inner.to_string(),
			DataEnum::Bool(inner) => inner.to_string(),
			_ => "".to_string()
		};
		let data = ParsedData {
			data: key.data, // Temporary Value
			name,
			need_delete: false
		};
		self.inner.push(data);
		Ok(())
	}

	fn serialize_value<T: ?Sized + Serialize>(&mut self, input: &T) -> Result<(), Error> {
		let parse = input.serialize(&mut Parser {})?;
		let len = self.inner.len() - 1;
		let key_data = self.inner[len].clone();
		self.inner[len] = ParsedData {
			name: key_data.name.clone(),
			data: DataEnum::Map(Box::new((key_data, parse))),
			need_delete: false,
		};
		Ok(())
	}

	fn end(self) -> Result<ParsedData, Error> { 
		Ok(ParsedData {
			name: self.final_name.clone(), 
			..self.inner.into()
		}) 
	}
}

impl ser::SerializeStruct for Layer {
	type Ok = ParsedData;
	type Error = Error;

	fn serialize_field<T>(&mut self, name: &'static str, value: &T)  -> Result<(), Error> 
	where
		T: ?Sized + Serialize,
	{
		self.inner.push(value.serialize(&mut Parser {})?);
		let len = self.inner.len() - 1;
		self.inner[len].name = name.into();
		Ok(())
	}

	fn end(self) -> Result<ParsedData, Error> { 
		Ok(ParsedData {
			name: self.final_name.clone(), 
			..self.inner.into()
		}) 
	}
}

impl ser::SerializeStructVariant for Layer {
	type Ok = ParsedData;
	type Error = Error;

	fn serialize_field<T>(&mut self, name: &'static str, value: &T)  -> Result<(), Error> 
	where
		T: ?Sized + Serialize,
	{
		self.inner.push(value.serialize(&mut Parser {})?);
		let len = self.inner.len() - 1;
		self.inner[len].name = name.into();
		Ok(())
	}

	fn end(self) -> Result<ParsedData, Error> { 
		Ok(ParsedData{
			data: DataEnum::Enum(self.final_name, self.inner.into()),
			name: "".to_string(),
			need_delete: false
		})
	}
}

macro_rules! deserialize {
	($i1: ident, $i2: ident,$s: tt , $t:ty) => {
		fn $i1<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
			if let DataEnum::$s(t) = &self.data.data {
				let value = input.$i2(t.clone() as $t)?;
				self.data.need_delete = true;
				Ok(value)
			}else {
				Err(Error::UnexpectedType(stringify!($t).to_string()))
			}
		}
	}
}

impl<'a, 'de> Deserializer<'de> for &'a mut DeParser<'_> {
	type Error = Error;
	deserialize!(deserialize_bool, visit_bool, Bool, bool);
	deserialize!(deserialize_i8, visit_i8, Int, i8);
	deserialize!(deserialize_i16, visit_i16, Int, i16);
	deserialize!(deserialize_i32, visit_i32, Int, i32);
	deserialize!(deserialize_i64, visit_i64, Int, i64);
	deserialize!(deserialize_u8, visit_u8, Int, u8);
	deserialize!(deserialize_u16, visit_u16, Int, u16);
	deserialize!(deserialize_u32, visit_u32, Int, u32);
	deserialize!(deserialize_u64, visit_u64, Int, u64);
	deserialize!(deserialize_f32, visit_f32, Float, f32);
	deserialize!(deserialize_f64, visit_f64, Float, f64);
	deserialize!(deserialize_string, visit_string, String, String);
	fn deserialize_any<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		match &self.data.data {
			DataEnum::Node(inner) => {
				let mut fields = vec!();
				if inner.len() == 1 {
					*self.data = inner[0].clone();
					return self.deserialize_any(input)
				}
				for data in inner {
					if data.name.is_empty() {
						return self.deserialize_seq(input)
					}else {
						fields.push(ParsedData {
							name: data.name.clone(),
							data: DataEnum::Map(Box::new((data.name.clone().into(), data.clone()))),
							need_delete: false,
						});
					}
				}
				*self.data = ParsedData {
					data: DataEnum::Node(fields),
					..Default::default()
				};
				self.deserialize_map(input)
			},
			DataEnum::Map(_) => self.deserialize_map(input),
			DataEnum::Enum(_, _) => self.deserialize_enum(&"", &[], input),
			DataEnum::Data(_) => self.deserialize_bytes(input),
			DataEnum::String(_) => self.deserialize_string(input),
			DataEnum::Int(_) => self.deserialize_i64(input),
			DataEnum::Float(_) => self.deserialize_f64(input),
			DataEnum::Bool(_) => self.deserialize_bool(input),
			DataEnum::None => self.deserialize_unit(input),
		}
	}

	fn deserialize_char<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::String(t) = &self.data.data {
			let value = input.visit_char(t.chars().next().unwrap())?;
			self.data.need_delete = true;
			Ok(value)
		}else {
			Err(Error::UnexpectedType(stringify!($t).to_string()))
		}
	}

	fn deserialize_str<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::String(t) = &self.data.data {
			let value = input.visit_str(&t)?;
			self.data.need_delete = true;
			Ok(value)
		}else {
			Err(Error::UnexpectedType(stringify!(str).to_string()))
		}
	}

	fn deserialize_bytes<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::Data(t) = &self.data.data {
			let value = input.visit_bytes(&t)?;
			self.data.need_delete = true;
			Ok(value)
		}else {
			Err(Error::UnexpectedType(stringify!(&[u8]).to_string()))
		}
	}

	fn deserialize_byte_buf<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::Data(t) = &self.data.data {
			let value = input.visit_byte_buf(t.to_vec())?;
			self.data.need_delete = true;
			Ok(value)
		}else {
			Err(Error::UnexpectedType(stringify!(&[u8]).to_string()))
		}
	}

	fn deserialize_option<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::None = self.data.data {
			input.visit_none()
		}else {
			input.visit_some(self)
		}
	}

	fn deserialize_unit<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::None = self.data.data {
			input.visit_unit()
		}else {
			Err(Error::UnexpectedType(stringify!(None).to_string()))
		}
	}

	fn deserialize_unit_struct<V: Visitor<'de>>(self,_:&'static str, input: V) -> Result<V::Value, Error> { self.deserialize_unit(input) }

	fn deserialize_newtype_struct<V: Visitor<'de>>(self,_:&'static str, input: V) -> Result<V::Value, Error> { self.deserialize_unit(input) }

	fn deserialize_seq<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::Node(vec) = &mut self.data.data {
			vec.retain(|data| !data.need_delete);
		}else {
			return Err(Error::UnexpectedType(stringify!(seq).to_string()));
		}
		input.visit_seq(DeLayer::from(self.data))
	}

	fn deserialize_map<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> {
		if let DataEnum::Node(_) = self.data.data {
		}else {
			return Err(Error::UnexpectedType(stringify!(seq).to_string()));
		}
		input.visit_map(DeMap::from(self.data))
	}

	fn deserialize_tuple_struct<V: Visitor<'de>>(self, _: &'static str, _: usize, input: V) -> Result<V::Value, Error> { self.deserialize_seq(input) }

	fn deserialize_tuple<V: Visitor<'de>>(self,_: usize, input: V) -> Result<V::Value, Error> { self.deserialize_seq(input) }

	fn deserialize_struct<V: Visitor<'de>>(self,_: &'static str, fields: &'static [&'static str], input: V) -> Result<V::Value, Error> {
		if let DataEnum::Node(vec) = &mut self.data.data {
			let mut output = vec!();
			for index in 0..vec.len() {
				output.push(ParsedData {
					data: DataEnum::Map(Box::new((fields[index].into(), vec[index].clone()))),
					name: String::new(),
					need_delete: false,
				});
			}
			*self.data = ParsedData {
				data: DataEnum::Node(output),
				name: String::new(),
				need_delete: self.data.need_delete
			}
		}else {
			return Err(Error::UnexpectedType(stringify!(struct).to_string()));
		}
		self.deserialize_map(input)
	}

	fn deserialize_enum<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], input: V) -> Result<V::Value, Error> {
		if let DataEnum::Enum(value, inner) = &self.data.data {
			if inner.is_empty() {
				return input.visit_enum(value.clone().into_deserializer());
			}else {
				input.visit_enum(DeEnum { 
					inner: &mut DeParser { data: self.data },
				})
			}
		}
		else { 
			return Err(Error::UnexpectedType(stringify!(enum).to_string())); 
		}
		
	}

	fn deserialize_identifier<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> { self.deserialize_any(input) }

	fn deserialize_ignored_any<V: Visitor<'de>>(self, input: V) -> Result<V::Value, Error> { self.deserialize_any(input) }
}

impl<'de> SeqAccess<'de> for DeLayer<'_> {
	type Error = Error;
	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error> 
	where
		T: DeserializeSeed<'de>,
	{	
		if let DataEnum::Node(vec) = &mut self.inner.data.data {
			vec.retain(|data| !data.need_delete);
			if vec.is_empty() {
				return Ok(None)
			}else {
				let len = vec.len() - 1;
				Ok(Some(seed.deserialize(&mut DeParser { data: &mut vec[len] })?))
			}
		}else {
			unreachable!()
		}
	}
}

impl<'de, 'a> MapAccess<'de> for DeMap<'_> {
	type Error = Error;
	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
	where 
		K: DeserializeSeed<'de>,
	{
		if let DataEnum::Node(vec) = &mut self.inner.data.data {
			vec.retain(|data| !data.need_delete);
			if vec.is_empty() {
				return Ok(None)
			}else {
				let len = vec.len() - 1;
				if let DataEnum::Map(box_inside) = &vec[len].data {
					let (mut key, value) = *box_inside.clone();
					self.temp = Some(value);
					Ok(Some(seed.deserialize(&mut DeParser { data: &mut key })?))
				}else {
					return Err(Error::UnexpectedType(stringify!(Map).to_string()));
				}
			}
		}else {
			unreachable!()
		}
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
	where 
		V: DeserializeSeed<'de>,
	{
		if let DataEnum::Node(vec) = &mut self.inner.data.data {
			let len = vec.len() - 1;
			vec[len].need_delete = true;
		}
		let mut temp = self.temp.clone().unwrap();
		seed.deserialize(&mut DeParser { data: &mut temp })
	}
}

impl<'de, 'a> EnumAccess<'de> for DeEnum<'a> {
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
	where
		V: DeserializeSeed<'de>
	{
		if let DataEnum::Enum(key, _) = &self.inner.data.data {
			let val = seed.deserialize(&mut DeParser { data: &mut key.clone().into() })?;
			Ok((val, self))
		}else {
			unreachable!()
		}
	}
}

impl<'de, 'a> VariantAccess<'de> for DeEnum<'a> {
	type Error = Error;

	fn unit_variant(self) -> Result<(), Error> {
		Err(Error::Syntax)
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
	where
		T: DeserializeSeed<'de>,
	{
		if let DataEnum::Enum(_, inner) = &self.inner.data.data {
			seed.deserialize(&mut DeParser {
				data: &mut inner.clone()[0]
			})
		}else {
			unreachable!()
		}
	}

	fn tuple_variant<V>(self, _len: usize, input: V) -> Result<V::Value, Error>
	where
		V: Visitor<'de>,
	{
		if let DataEnum::Enum(_, inner) = &self.inner.data.data {
			DeParser {
				data: &mut ParsedData {
					data: DataEnum::Node(inner.clone()),
					..Default::default()
				}
			}.deserialize_seq(input)
		}else {
			unreachable!()
		}
	}

	fn struct_variant<V>(self, _fields: &'static [&'static str], input: V) -> Result<V::Value, Error>
	where
		V: Visitor<'de>,
	{
		if let DataEnum::Enum(_, inner) = &self.inner.data.data {
			DeParser {
				data: &mut ParsedData {
					data: DataEnum::Node(inner.clone()),
					..Default::default()
				}
			}.deserialize_any(input)
		}else {
			unreachable!()
		}
	}
}