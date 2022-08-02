#![allow(dead_code)]
use std::fmt;

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
    data: Vec<Record>,
}

impl Table {
    /// Create new table
    fn new(name: String) -> Table {
        return Table {
            name: name,
            data: Vec::new(),
        }
    }

    /// Get name of the table
    pub fn get_name(&self) -> &str {
        return &self.name[..];
    }

    /// Create new row or update currently exists record
    pub fn insert_or_update(&mut self, key: &str, value: &str) {
        let record = Record::new(String::from(key), String::from(value));        
        match self.find_next_index(key) {
            Some(index) => {
                self.data.insert(index, record)
            },
            None => {
                self.data.push(record);
            }
        }
    }

    /// Filter table records based on an input function
    pub fn filter<F>(&self, key_filter: F) -> Vec<Record> 
    where 
        F: Fn(&Record) -> bool, 
    {
        let mut collected: Vec<Record> = Vec::new();

        for record in &self.data {
            if key_filter(record) {
                collected.push(record.clone());
            }
        }

        return collected;
    }

    /// Remove the selected elements
    pub fn remove<F>(&mut self, remove_filter: F) -> Option<usize> 
    where F: Fn(&Record) -> bool {
        let mut remove_list: Vec<usize> = Vec::new();

        let mut index: usize = 0;
        for record in &self.data {
            if remove_filter(record) {
                remove_list.push(index);
            }
            index += 1;
        }

        if remove_list.len() == 0 {
            return None;
        }

        index = 0;
        for i in (0..remove_list.len()).rev() {
            self.data.remove(remove_list[i]);
            index += 1;
        }

        return Some(index);
    }

    /// Convert the Record into another type
    pub fn select<F, T>(&self, select_func: F) -> Vec<T> 
    where 
        F: Fn(&Record) -> Option<T>,
    {
        let mut result: Vec<T> = Vec::new();

        for record in &self.data {
            if let Some(output) = select_func(record) {
                result.push(output);
            }
        }

        return result;
    }

    fn find_next_index(&self, name: &str) -> Option<usize> {
        let mut low: i32 = 0;
        let mut high: i32 = (self.data.len() as i32) - 1;
        let mut mid: i32 = low + ((high - low) / 2);

        while low <= high {
            mid = low + ((high - low) / 2);
            let mid_value = &self.data[mid as usize];

            if mid_value.get_key() < name {
                low = mid + 1;
            }
            else if mid_value.get_key() > name {
                high = mid - 1;
            }
            else {
                return Some(mid as usize);
            }
        }

        return Some(mid as usize);
    }

    fn find_record_index(&self, name: &str) -> Option<usize> {
        let mut low: i32 = 0;
        let mut high: i32 = (self.data.len() as i32) - 1;

        while low <= high {
            let mid: i32 = low + ((high - low) / 2);
            let mid_value = &self.data[mid as usize];

            if mid_value.get_key() < name {
                low = mid + 1;
            }
            else if mid_value.get_key() > name {
                high = mid - 1;
            }
            else {
                return Some(mid as usize);
            }
        }

        return None;
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_text = String::new();
        for record in &self.data {
            final_text += record.get_key();
            final_text += ";";
            final_text += record.get_value();
            final_text += ";";
            final_text += "\n";
        }
        return write!(f, "{}", final_text);
    }
}

/// Record structure
/// 
/// A record consist from a `key` and a `values`
#[derive(Clone)]
pub struct Record {
    key: String,
    value: String,
}

impl Record {
    fn new(key: String, value: String) -> Record {
        return Record { key: key, value: value }
    }

    pub fn get_key(&self) -> &str {
        return &self.key[..];
    }

    pub fn get_value(&self) -> &str {
        return &self.value[..];
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{};{};", self.key, self.value);
    }
}
