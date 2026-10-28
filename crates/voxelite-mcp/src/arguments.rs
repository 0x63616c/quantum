//! Typed wire fields and schema generation for the private tool registry.
//!
//! Schema validation always runs first. Fields retain their wire representation
//! until the handler reads them: JSON Schema accepts `1.0` as an integer while
//! serde's integer decoder rejects it. Eager decoding would change error paths,
//! precedence (including revision checks), and ignored query pagination fields.
use crate::error::{Failure, Result};
use schemars::{JsonSchema, SchemaGenerator};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::marker::PhantomData;

pub(crate) struct Field<T> {
    value: Option<Value>,
    name: &'static str,
    marker: PhantomData<T>,
}
impl<T: DeserializeOwned> Field<T> {
    pub fn new(value: &Value, name: &'static str) -> Self {
        Self {
            value: value.get(name).cloned(),
            name,
            marker: PhantomData,
        }
    }
    pub fn get(&self) -> Result<T> {
        serde_json::from_value(self.value.clone().unwrap_or(Value::Null))
            .map_err(|e| Failure::invalid(e.to_string()).at(format!("/{}", self.name)))
    }
    pub fn optional(&self) -> Result<Option<T>> {
        self.value.as_ref().map(|_| self.get()).transpose()
    }
    pub fn default(&self, value: T) -> Result<T> {
        Ok(self.optional()?.unwrap_or(value))
    }
    pub fn is_present(&self) -> bool {
        self.value.is_some()
    }
}

pub(crate) trait Input: DeserializeOwned + JsonSchema {
    fn fields() -> Vec<crate::catalog::Field>;
}
pub(crate) fn schema<T: JsonSchema>() -> Value {
    T::json_schema(&mut SchemaGenerator::default()).to_value()
}

// A field's wire name, Rust type, optionality and constraints are declared once.
// The macro emits both serde decoding and JsonSchema, preserving the v1 field
// order and inline schemas (including empty `required` arrays).
macro_rules! arguments {
    ($name:ident { $($kind:ident $field:ident : $ty:ty => $schema:expr,)* } $(where $constraint:path)?) => {
        pub(crate) struct $name { $(pub $field: arguments!(@type $kind $ty),)* }
        impl From<&serde_json::Value> for $name {
            fn from(value: &serde_json::Value) -> Self {
                let _ = value;
                Self { $($field: arguments!(@parse $kind value $field $ty),)* }
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self::from(&value))
            }
        }
        impl crate::arguments::Input for $name {
            fn fields() -> Vec<crate::catalog::Field> {
                let fields: Vec<Vec<crate::catalog::Field>> = vec![$(arguments!(@fields $kind $field $ty => $schema)),*];
                fields.into_iter().flatten().collect()
            }
        }
        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> { stringify!($name).into() }
            fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
                let value = crate::catalog::object(<Self as crate::arguments::Input>::fields());
                $(let value = { let mut value = value; $constraint(&mut value); value };)?
                value.try_into().expect("built-in argument schema")
            }
        }
    };
    (@type flat $ty:ty) => { $ty };
    (@type $kind:ident $ty:ty) => { crate::arguments::Field<$ty> };
    (@parse flat $value:ident $field:ident $ty:ty) => { <$ty>::from($value) };
    (@parse $kind:ident $value:ident $field:ident $ty:ty) => { crate::arguments::Field::new($value, stringify!($field)) };
    (@fields flat $field:ident $ty:ty => $schema:expr) => { <$ty as crate::arguments::Input>::fields() };
    (@fields req $field:ident $ty:ty => $schema:expr) => { vec![crate::catalog::req(stringify!($field), $schema)] };
    (@fields opt $field:ident $ty:ty => $schema:expr) => { vec![crate::catalog::opt(stringify!($field), $schema)] };
}
pub(crate) use arguments;

use crate::catalog::{bounded_name, revision, uuid};
use voxelite_core::Uuid;
arguments!(Identity {
    req expected_epoch: Uuid => uuid(),
    req operation_id: Uuid => uuid(),
});
arguments!(NewCommon {
    flat identity: Identity => (),
    opt label: String => bounded_name(80),
});
arguments!(ReadCommon {
    req model_id: Uuid => uuid(),
    opt revision: String => revision(),
});
arguments!(WriteCommon {
    flat identity: Identity => (),
    req model_id: Uuid => uuid(),
    req expected_revision: String => revision(),
    opt label: String => bounded_name(80),
});
