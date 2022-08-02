#![allow(dead_code)]
use std::fmt;
use std::collections::BTreeMap;
use std::ops::Bound::Included;

/// Database struture
/// 
/// This is the primary structure for database. A Database consist of set of tables.
/// Database implementation contains actions with tables like: create, drop, select, etc.
pub struct Database {
    tables: Vec<Table>,
}

impl Database {
    /// Initialize a new `Database` instance
    pub fn new() -> Database {
        return Database { 
            tables: Vec::new(),
        };
    }

    /// Return with an iterable vector about tables
    pub fn get_tables(&self) -> &Vec<Table> {
        return &self.tables;
    }

    /// Filter tables based on input function
    pub fn filter_tables<F>(&self, filter_func: F) -> Vec<&Table>
    where
        F: Fn(&str) -> bool,
    {
        let mut collected: Vec<&Table> = Vec::new();

        for table in &self.tables {
            if filter_func(table.get_name()) {
                collected.push(table);
            }
        }

        return collected;
    }

    /// Create new table
    pub fn create_table(&mut self, name: String) -> Result<(), String> {
        if let Some(_) = self.find_table(&name[..]) {
            return Err(String::from("Table already exist"));
        }

        let mut index: usize = 0;
        for table in &self.tables {
            if name < table.name {
                break;
            }
            index += 1;
        }
        self.tables.insert(index, Table::new(name));
        return Ok(());
    }

    /// Find specific table based on its name
    pub fn select_table(&self, name: &str) -> Option<&Table> {
        match self.find_table(name) {
            Some(index) => Some(&self.tables[index]),
            None => None,
        }
    }

    /// Find a specific table and return based its name as a mutable reference
    pub fn select_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        match self.find_table(name) {
            Some(index) => Some(&mut self.tables[index]),
            None => None,
        }
    }

    /// Drop table and all its content by name
    pub fn drop_table(&mut self, name: &str) -> Result<(), String> {
        match self.find_table(name) {
            Some(index) => {
                self.tables.remove(index);
                return Ok(());
            },
            None => Err(format!("Table ({}) did not found", name)),
        }
    }

    /// Clear the current table content and copy from different
    pub fn copy_table(&mut self, source: &str, target: &str) -> Result<(), String> {
        let source = match self.find_table(source) {
            Some(index) => index,
            None => return Err(String::from("Source table does not exist")),
        };

        let target = match self.find_table(target) {
            Some(index) => index,
            None => return Err(String::from("Target table does not exist")),
        };

        self.tables[target].data = self.tables[source].data.clone();

        return Ok(());
    }

    /// Find table, internal function
    fn find_table(&self, name: &str) -> Option<usize> {
        let mut low: i32 = 0;
        let mut high: i32 = (self.tables.len() as i32) - 1;

        while low <= high {
            let mid: i32 = low + ((high - low) / 2);
            let mid_value = &self.tables[mid as usize];

            if mid_value.get_name() < name {
                low = mid + 1;
            }
            else if mid_value.get_name() > name {
                high = mid - 1;
            }
            else {
                return Some(mid as usize);
            }
        }

        return None;
    }

}

/// Table structure
/// 
/// A table consist of `Record` elements.
pub struct Table {
    name: String,
    data: BTreeMap<String, String>,
}

impl Table {
    /// Create new table
    fn new(name: String) -> Table {
        return Table {
            name: name,
            data: BTreeMap::new(),
        }
    }

    /// Get name of the table
    pub fn get_name(&self) -> &str {
        return &self.name[..];
    }

    /// Create new row or update currently exists record
    pub fn insert_or_update(&mut self, key: &str, value: &str) {
        self.data.insert(String::from(key), String::from(value));
    }

    /// Get value for specific key
    pub fn get_value(&self, key: &str) -> Option<&String> {
        return self.data.get(key);
    }

    pub fn key_start_with(&self, start_with: &str) -> Vec<&String> {
        let mut collected: Vec<&String> = Vec::new();

        let start_with1 = String::from(start_with);
        let start_with2 = String::from(start_with);
        for record in self.data.range(start_with1..) {
            if start_with2.len() <= record.0.len() {
                let len = start_with2.len();
                if &record.0[0..len] != &start_with2[..] {
                    break;
                }
            }
            else {
                break;
            }
            collected.push(record.0);
        }

        return collected;
    }

    /// Filter table records based on an input function
    pub fn filter_keys<F>(&self, key_filter: F) -> Vec<&String> 
    where 
        F: Fn(&String) -> bool,
    {
        let mut collected: Vec<&String> = Vec::new();

        for record in &self.data {
            if key_filter(record.0) {
                collected.push(record.0);
            }
        }

        return collected;
    }

    /// Remove the selected elements
    pub fn remove<F>(&mut self, remove_filter: F) -> Option<usize> 
    where F: Fn(&String) -> bool {
        let mut remove_list: Vec<String> = Vec::new();

        let mut index: usize = 0;
        for record in &mut self.data {
            if remove_filter(record.0) {
                remove_list.push(record.0.clone());
            }
            index += 1;
        }

        if remove_list.len() == 0 {
            return None;
        }

        index = 0;
        for key in remove_list {
            self.data.remove(&key);
            index += 1;
        }

        return Some(index);
    }

    /// Convert the Record into another type
    pub fn select<F, T>(&self, select_func: F) -> Vec<T> 
    where 
        F: Fn(&String, &String) -> Option<T>,
    {
        let mut result: Vec<T> = Vec::new();

        for record in &self.data {
            if let Some(output) = select_func(record.0, record.1) {
                result.push(output);
            }
        }

        return result;
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_text = String::new();
        for record in &self.data {
            final_text += record.0;
            final_text += ";";
            final_text += record.1;
            final_text += ";";
            final_text += "\n";
        }
        return write!(f, "{}", final_text);
    }
}
