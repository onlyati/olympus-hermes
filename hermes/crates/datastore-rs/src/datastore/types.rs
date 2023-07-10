//! Custom types

use std::collections::BTreeMap;
use super::enums::error::ErrorKind;

use super::enums::pair::{KeyType, ValueType};

pub type Table = BTreeMap<KeyType, ValueType>;

pub type ResultWithResult = Result<ValueType, ErrorKind>;
pub type ResultWithoutResult = Result<(), ErrorKind>;
pub type ResultWithList = Result<Vec<KeyType>, ErrorKind>;
pub type ResultWithHook = Result<(String, Vec<String>), ErrorKind>;
pub type ResultWithHooks = Result<BTreeMap<String, Vec<String>>, ErrorKind>;
