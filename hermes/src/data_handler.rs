use std::collections::HashMap;
use std::collections::LinkedList;
use chrono::{Datelike, Timelike, Utc};

pub struct Group
{
    content: HashMap<String, Item>,
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
    pub fn new() -> Group
    {
        Group
        {
            content: HashMap::new(),
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
    /// It returns a Tuple which is within Option. If no key was found, it returns with None, else it returns with Some<(String, String, String)>.
    pub fn find(&self, item_name: &str) -> Option<(String, String, String)>
    {
        match self.content.get(item_name) {
            Some(v) => return Option::Some((item_name.to_string(), v.last_update.clone(), v.content.clone())),
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
    /// Returns with a vector which contains a String elements.
    pub fn filter(&self, name_chunk: &str) -> Option<Vec<String>>
    {
        let mut output_list: LinkedList<String> = LinkedList::new();

        let chunk_length = name_chunk.len();
        let mask: &str = "*";

        if name_chunk == mask {
            // List everything
            for item in self.content.iter() {
                output_list.push_back(item.0.to_string());
            }
            return convert_to_vec_string(output_list);
        }

        if &name_chunk[0..1] == mask && &name_chunk[chunk_length - 1..chunk_length] == mask {
            let mut name_chunk_reduced: &str = &name_chunk[..chunk_length - 1];
            name_chunk_reduced = &name_chunk_reduced[1..];
            // List contains
            for item in self.content.iter() {
                if item.0.contains(name_chunk_reduced) {
                    output_list.push_back(item.0.to_string());
                }
            }
            return convert_to_vec_string(output_list);
        }

        if &name_chunk[0..1] == mask {
            // Begin is wildcarded
            let name_chunk_reduced = &name_chunk[1..];
            let name_chunk_reduced_size = name_chunk_reduced.len();
            for item in self.content.iter() {
                let item_len = item.0.len();
                if name_chunk_reduced_size > item_len {
                    continue;
                }
                let start_size = item_len - name_chunk_reduced_size;
                if &item.0[start_size..] == name_chunk_reduced {
                    output_list.push_back(item.0.to_string());
                }
            }
            return convert_to_vec_string(output_list);
        }

        if &name_chunk[chunk_length - 1..chunk_length] == mask {
            // End is wildcarded
            let name_chunk_reduced: &str = &name_chunk[..chunk_length - 1];
            let name_chunk_reduced_size = name_chunk_reduced.len();
            for item in self.content.iter() {
                if &item.0[0..name_chunk_reduced_size] == name_chunk_reduced {
                    output_list.push_back(item.0.to_string());
                }
            }
            return convert_to_vec_string(output_list);
        }

        match self.content.get(name_chunk) {
            Some(_) => {
                output_list.push_back(String::from(name_chunk));
                return convert_to_vec_string(output_list);
            },
            None => return None,
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
    pub fn insert_or_update(&mut self, item_name: &str, value: &str) -> Option<String> {
        if let Some(v) = self.content.get(item_name) {
            if v.content == value.to_string() {
                return None;
            }
        }

        let now = Utc::now();
        let time_now = format!("{}-{}-{} {}:{}:{}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
        self.content.insert(item_name.to_string(), Item::new(String::from(time_now), String::from(value)));

        return Some(format!("Item ({}) is added", item_name));
    }

    /// Delete item
    /// 
    /// This function try to delete from the group
    /// 
    /// # Input(s):__rust_force_expr!
    /// 
    /// - Name of the item
    /// 
    /// # Return
    /// 
    /// Result, depends that remove was successful or not
    pub fn delete(&mut self, item_name: &str) -> Option<String> {
        match self.content.remove(item_name) {
            Some(_) => return Some(format!("Item ({}) is deleted", item_name)),
            None => return None,
        }
    }
}

struct Item
{
    last_update: String,
    content: String,
}

impl Item
{
    fn new(date_time_now: String, content: String) -> Item
    {
        Item
        {
            last_update: date_time_now,
            content: content,
        }
    }
}

fn convert_to_vec_string(list: LinkedList<String>) -> Option<Vec<String>> {
    let mut output: Vec<String> = Vec::with_capacity(list.len());

    let mut i: i32 = 0;
    for item in list {
        output.push(item);
        i = i + 1;
    }

    match output.len() {
        0 => Option::None,
        _ => Some(output),
    }
}