use std::collections::HashMap;

use onlyati_http::parser::HttpResponse;
use onlyati_http::parser::RequestInfo;
use onlyati_http::parser::RequestResponse;

use crate::DATA;

/// Add new group
/// 
/// This is called for a POST request. This function create a new group, if it does not already exist.
/// 
/// ## HTTP variables:
/// 
/// - `group`: Name of the group where the item will be searched
/// 
/// ## HTTP returns:
/// 
/// - `BadRequest`: If `group` is missing or group already exist
/// - `InternalServerError`: If there are some issue, e.g.: problem with Mutex lock
/// - `Ok`: Group is successfully added
pub fn add_group(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Save the name of the group
    let group: String;
    match info.parameters.get("name") {
        Some(r) => group = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut answer: String;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.add_group(&group[..]) {
                    Some(v) => answer = v,
                    None => return RequestResponse::new(HttpResponse::BadRequest, header, format!("Specified group ({}) is already exist", group)),
                }
            }
            return RequestResponse::new(HttpResponse::Ok, header, answer);
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Drop group
/// 
/// This is called for a DELETE request. This function delete a whole group if it does exist.
/// 
/// ## HTTP variables:
/// 
/// - `group`: Name of the group where the item will be searched
/// 
/// ## HTTP returns:
/// 
/// - `BadRequest`: If `group` is missing
/// - `InternalServerError`: If there are some issue, e.g.: problem with Mutex lock
/// - `NotFound`: Specified group does not exist
/// - `Ok`: Group is successfully added
pub fn drop_group(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Save the name of the group
    let group: String;
    match info.parameters.get("name") {
        Some(r) => group = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut answer: String;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.drop_group(&group[..]) {
                    Some(v) => answer = v,
                    None => return RequestResponse::new(HttpResponse::NotFound, header, format!("Group ({}) does not exist", group)),
                }
            }
            return RequestResponse::new(HttpResponse::Ok, header, answer);
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// List all groups
/// 
/// This is called for a GET request. This function list all exisiting groups
/// 
/// ## HTTP variables:
/// 
/// - `group`: Name of the group where the item will be searched
/// 
/// ## HTTP returns:
/// 
/// - `InternalServerError`: If there are some issue, e.g.: problem with Mutex lock
/// - `Ok`: Group is successfully added
pub fn list_group(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut list: Vec<String>;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.list_all_group() {
                    Some(v) => list = v,
                    None => list = Vec::new(),
                }
            }

            let mut answer: String = format!("{}\n", list.len());

            if list.len() > 0 {
                for i in list {
                    answer = answer + i.as_str() + "\n";
                }
            }

            return RequestResponse::new(HttpResponse::Ok, header, answer);
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}