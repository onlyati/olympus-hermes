use std::collections::HashMap;

use onlyati_http::parser::HttpResponse;
use onlyati_http::parser::RequestInfo;
use onlyati_http::parser::RequestResponse;

use crate::DATA;

/// Delete value
/// 
/// This is called for DELETE /item/remove?name=xxxxxx request.
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

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut answer: String;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.delete_from_group(&name[..]) {
                    Some(v) => answer = v,
                    None => answer = String::from("Key was not exist"),
                }
                return RequestResponse::new(HttpResponse::Ok, header, answer);
            }
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Set value
/// 
/// This is called for POST /item/set?name=xxxxx request. Value of the key is in the `info.body`
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
            let mut answer: String = String::new();
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.insert_or_update(&name[..], &value[..]) {
                    Some(v) => answer = v.clone(),
                    None => answer = String::from("Key already has this value, no update"),
                }
            }
            return RequestResponse::new(HttpResponse::Ok, header, answer);
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

/// Get value
/// 
/// This is called for GET /item/get?name=xxxx request. It returns the value of the key.
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

    // Try to find data, set response accordingly
    let data_mut = DATA.get();
    match data_mut {
        Some(r) => {
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.find(&name[..]) {
                    Some(r) => {
                        let resp: String = format!("{}\n{}\n{}\n", r.1, r.0, r.2);
                        return RequestResponse::new(HttpResponse::Ok, header, resp);
                    },
                    None => return RequestResponse::new(HttpResponse::NotFound, header, String::from("Key was not found")),
                }
            }
        },
        None => return RequestResponse::new(HttpResponse::InternalServerError, header, String::from("Sorry :-(")),
    }
}

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

    let data_mut = DATA.get();
    match data_mut {
        Some(_) => {
            let mut list: Vec<String>;
            {
                let mut data = data_mut.unwrap().lock().unwrap();
                match data.filter(&filter[..]) {
                    Some(v) => list = v,
                    None => list = Vec::new(),
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