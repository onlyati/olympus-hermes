use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use chrono::{Datelike, Timelike, Utc};

pub struct Group
{
    name: String,
    content: Arc<Mutex<HashMap<String, Item>>>,
}

impl Group
{
    /// Create new group structure
    ///
    /// This function initialize a new structure
    ///
    /// # Input(s):
    ///
    /// - Name of the group
    ///
    /// # Return value:
    ///
    /// Group sturcture.
    pub fn new(name: String) -> Group
    {
        Group
        {
            name: name,
            content: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Find element in the hashmap
    ///
    /// This function will looking for specified entry in the HashMap
    ///
    /// # Input(s):
    ///
    /// - Key which must be found in hashmap
    ///
    /// # Return:
    ///
    /// It returns a Tuple which is within Option. If no key was found, it returns with None, else it returns with Some<(String, String)>.
    pub fn find(&self, item_name: &str) -> Option<Item>
    {
        let list = self.content.clone();
        let list = match list.lock() {
            Ok(guard) => guard,
            Err(posion) => posion.into_inner(),
        };

        match list.get(item_name) {
            Some(v) => return Option::Some(Item::new(String::from(&v.last_update[..]), String::from(&v.content[..]))),
            None => return Option::None,
        }
    }

    /// Find elements based
    ///
    /// This function return with a list about those items which name contains the specified chunk.
    ///
    /// # Input(s):
    ///
    /// - Text chunk which needs to be found in key.
    ///
    /// # Return value:
    ///
    /// Returns with a vector which contains a tuple `(String, Item)`.
    pub fn filter(&self, name_chunk: &str) -> Option<Vec<(String, Item)>>
    {
        let list = self.content.clone();
        let list = match list.lock() {
            Ok(guard) => guard,
            Err(posion) => posion.into_inner(),
        };

        let mut output: Vec<(String, Item)> = Vec::new();

        for item in list.iter()
        {
            if item.0.contains(name_chunk)
            {
                output.push((item.0.to_string(), Item::new(String::from(item.1.last_update.as_str()), String::from(item.1.content.as_str()))));
            }
        }

        match output.len() {
            0 => Option::None,
            _ => Some(output),
        }
    }

    /// Insert new element
    ///
    /// This function insert or update item in Group sturcture. Date and time automatically filled.
    ///
    /// # Input(s):
    ///
    /// - Name of the item
    /// - Value of the item
    pub fn insert_or_update(&self, item_name: &str, value: &str) -> bool
    {
        let now = Utc::now();

        let time_now = format!("{}-{}-{} {}:{}:{}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());

        let list = self.content.clone();
        let mut list = match list.lock() {
            Ok(guard) => guard,
            Err(posion) => posion.into_inner(),
        };

        list.insert(item_name.to_string(), Item::new(String::from(time_now), String::from(value)));

        return true;
    }
}

pub struct Item
{
    last_update: String,
    content: String,
}

impl Item
{
    pub fn new(date_time_now: String, content: String) -> Item
    {
        Item
        {
            last_update: date_time_now,
            content: content,
        }
    }
}