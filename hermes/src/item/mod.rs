use std::collections::HashMap;

use onlyati_http::parser::HttpResponse;
use onlyati_http::parser::RequestInfo;
use onlyati_http::parser::RequestResponse;

use crate::DATA;
use crate::debug;

/// Delete value
/// 
/// This is called for DELETE request. Function delete the requested record from the specified group.
/// 
/// ## HTTP variables:
/// 
/// - `name`: Name of the item in group
/// - `group`: Name of the group where the item will be searched
/// 
/// ## HTTP returns:
/// 
/// - `BadRequest`: If `name` or `group` is/are missing
/// - `InternalServerError`: If there are some issue, e.g.: problem with Mutex lock
/// - `Ok`: If key was successfully removed
pub fn remove_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Save the name of the key
    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    // Save the name of the group
    let group: String;
    match info.parameters.get("group") {
        Some(r) => group = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: group")),
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut answer: String;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.get_group(&group[..]) {
                    Some(grp) => {
                        match grp.delete_from_group(&name[..]) {
                            Some(v) => answer = v,
                            None => return RequestResponse::new(HttpResponse::NotAcceptable, header, format!("Specified item ({}) does not exist", name))
                        }
                    },
                    None => return RequestResponse::new(HttpResponse::NotFound, header, format!("Specified group ({}) does not exist", group)),
                }
            }
            return RequestResponse::new(HttpResponse::Ok, header, answer);
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Set value
/// 
/// This is called for POST request. This function add new item onto Group structure.
/// 
/// ## HTTP variables:
/// 
/// - `name`: Name of the item in group
/// - `group`: Name of the group where the item will be searched
/// 
/// ## HTTP returns:
/// 
/// - `BadRequest`: If `name` or `group` is/are missing
/// - `InternalServerError`: If there are some issue, e.g.: problem with Mutex lock
/// - `NotFound`: If specified group does not exist
/// - `Ok`: If key was successfully added
pub fn set_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Save the name of the key
    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    // Save the name of the group
    let group: String;
    match info.parameters.get("group") {
        Some(r) => group = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: group")),
    }

    // Save the value from the body
    let value: String;
    if !info.body.trim().is_empty() {
        value = String::from(info.body.trim());
    }
    else {        
        return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing in body: value of key"));
    }

    // Try to insert incoming data and assemble the response accordingly
    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut answer: String;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.get_group(&group[..]) {
                    Some(grp) => {
                        match grp.insert_or_update(&name[..], &value[..]) {
                            Some(v) => answer = v,
                            None => return RequestResponse::new(HttpResponse::Ok, header, format!("Specified key ({}->{}) Already exist with same value, no update", group, name)),
                        }
                    },
                    None => return RequestResponse::new(HttpResponse::NotFound, header, format!("Specified group ({}) does not exist", group)),
                }
            }
            return RequestResponse::new(HttpResponse::Ok, header, answer);
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Get value
/// 
/// This is called for GET request. It returns with all data of single item: last change date, item name, item content
/// 
/// ## HTTP variables:
/// 
/// - `name`: Name of the item in group
/// - `group`: Name of the group where the item will be searched
/// 
/// ## HTTP returns:
/// 
/// - `BadRequest`: If `name` or `group` is/are missing
/// - `InternalServerError`: If there are some issue, e.g.: problem with Mutex lock
/// - `NotFound`: Specified item does not exist
/// - `Ok`: Key is found, return with all details
pub fn get_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Get key name from parameters
    let name: String;
    match info.parameters.get("name") {
        Some(r) => name = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: name")),
    }

    // Save the name of the group
    let group: String;
    match info.parameters.get("group") {
        Some(r) => group = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: group")),
    }

    // Try to find data, set response accordingly
    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.get_group(&group[..]) {
                    Some(grp) => {
                        match grp.find(&name[..]) {
                            Some(r) => {
                                let resp: String = format!("{}\n{}\n{}\n", r.1, r.0, r.2);
                                return RequestResponse::new(HttpResponse::Ok, header, resp);
                            },
                            None => return RequestResponse::new(HttpResponse::NoContent, header, String::from("")),
                        }
                    },
                    None => return RequestResponse::new(HttpResponse::NoContent, header, String::from("")),
                }
            }
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// List items in a group
/// 
/// This is a reaction for a GET request. It return with the item name list which is proper for the filter.
/// 
/// ## HTTP variables:
/// 
/// - `name`: mask for filter
/// - `group`: Name of the group where the item will be searched
/// 
/// ## HTTP returns:
/// 
/// - `BadRequest`: If `name` or `group` is/are missing
/// - `InternalServerError`: If there are some issue, e.g.: problem with Mutex lock
/// - `Ok`: Return with item name list. First line is the found number of names, then each line contain one name
pub fn filter_value(info: &RequestInfo) -> RequestResponse {
    // Response will be plain text
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert(String::from("Content-Type"), String::from("plain/text"));

    // Get the filter value
    let filter: String;
    match info.parameters.get("name") {
        Some(r) => filter = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing paramter: name")),
    }

    // Save the name of the group
    let group: String;
    match info.parameters.get("group") {
        Some(r) => group = String::from(r),
        None => return RequestResponse::new(HttpResponse::BadRequest, header, String::from("Missing parameter: group")),
    }

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut list: Vec<String>;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.get_group(&group[..]) {
                    Some(grp) => {
                        match grp.filter(&filter[..]) {
                            Some(v) => list = v,
                            None => list = Vec::new(),
                        }
                    },
                    None => return RequestResponse::new(HttpResponse::NotFound, header, format!("Specified group ({}) does not exist", group)),
                }
            }

            let mut answer: String = format!("{}\n", list.len());
            for key in list {
                answer = answer + &key[..] + "\n";
            }

            return RequestResponse::new(HttpResponse::Ok, header, answer);

        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}