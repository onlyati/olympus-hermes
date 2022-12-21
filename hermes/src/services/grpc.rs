use tonic::{transport::Server, Request, Response, Status};

use hermes::hermes_server::{Hermes, HermesServer};
use hermes::{SetPair, Key, TableName, Empty, Pair, KeyList, Table, TableList};

use crate::DB;

mod hermes {
    tonic::include_proto!("hermes");
}

#[derive(Debug, Default)]
struct HermesGrpc {}

#[tonic::async_trait]
impl Hermes for HermesGrpc {
    /// ## Set key value pair
    /// 
    /// This function execute set action via gRPC call
    async fn set(&self, request: Request<SetPair>) -> Result<Response<Pair>, Status> {
        let pair = request.into_inner();

        let db = DB.read().unwrap();
        let db = match &*db {
            Some(db) => db,
            None => return Err(Status::internal(String::from("Database is not available"))),
        };

        let table = match db.select_table(pair.table.as_str()) {
            Some(table) => table,
            None => return Err(Status::not_found(String::from("Table does not exist"))), 
        };

        table.insert_or_update(pair.key.as_str(), &pair.value.as_str());

        let value = match table.get_value(pair.key.as_str()) {
            Some(value) => value,
            None => return Err(Status::internal(String::from("Failed to create record"))),
        };

        if value != pair.value {
            return Err(Status::internal(String::from("Failed to update record")));
        }

        let result: Pair = Pair {
            key: pair.key,
            value: pair.value,
            table: pair.table,
        };
        return Ok(Response::new(result));
    }

    /// ## Get key
    /// 
    /// This function get the value of the key via gRPC call
    async fn get(&self, request: Request<Key>) -> Result<Response<Pair>, Status> {
        let key = request.into_inner();

        let db = DB.read().unwrap();
        let db = match &*db {
            Some(db) => db,
            None => return Err(Status::internal(String::from("Database is not available"))),
        };

        let table = match db.select_table(key.table.as_str()) {
            Some(table) => table,
            None => return Err(Status::not_found(String::from("Table does not exist"))), 
        };

        let value = match table.get_value(key.key.as_str()) {
            Some(value) => value,
            None => return Err(Status::not_found(String::from("Key does not exist in table"))),
        };

        let result = Pair {
            key: key.key,
            value: value,
            table: key.table,
        };
        return Ok(Response::new(result));
    }

    /// ## Delete key
    /// 
    /// This function delete key from specific table via gRPC call
    async fn delete(&self, request: Request<Key>) -> Result<Response<Pair>, Status> {
        let key = request.into_inner();

        let db = DB.read().unwrap();
        let db = match &*db {
            Some(db) => db,
            None => return Err(Status::internal(String::from("Database is not available"))),
        };

        let table = match db.select_table(key.table.as_str()) {
            Some(table) => table,
            None => return Err(Status::not_found(String::from("Table does not exist"))), 
        };

        let value = match table.get_value(key.key.as_str()) {
            Some(value) => value,
            None => return Err(Status::not_found(String::from("Key does not exist in table"))),
        };

        if let None = table.remove_key(key.key.as_str()) {
            return Err(Status::not_found(String::from("Key does not exist in table")));
        }

        let result = Pair {
            key: key.key,
            value: value,
            table: key.table,
        };
        return Ok(Response::new(result));
    }

    /// ## Mask keys
    /// 
    /// This function returns with those keys which are starts with specified key chunck via gRPC call
    async fn mask(&self, request: Request<Key>) -> Result<Response<KeyList>, Status> {
        let key = request.into_inner();

        let db = DB.read().unwrap();
        let db = match &*db {
            Some(db) => db,
            None => return Err(Status::internal(String::from("Database is not available"))),
        };

        let table = match db.select_table(key.table.as_str()) {
            Some(table) => table,
            None => return Err(Status::not_found(String::from("Table does not exist"))), 
        };

        let mut keys: Vec<String> = Vec::new();

        for key in table.key_start_with(key.key.as_str()) {
            keys.push(key);
        }

        let result = KeyList {
            keys: keys,
        };
        return Ok(Response::new(result));
    }

    /// ## Create table
    /// 
    /// This function create a table via gRPC call    
    async fn create_table(&self, request: Request<TableName>) -> Result<Response<Table>, Status> {
        let req_table = request.into_inner();
        let table = Table {
            name: req_table.name.clone(),
        };

        let mut db = DB.write().unwrap();
        let db = match &mut *db {
            Some(db) => db,
            None => return Err(Status::internal(String::from("Database is not available"))),
        };

        if let Some(_) = db.select_table(req_table.name.as_str()) {
            return Err(Status::already_exists(String::from("Specified table already exist")));
        }

        match db.create_table(req_table.name) {
            Ok(_) => return Ok(Response::new(table)),
            Err(e) => return Err(Status::internal(format!("Failed to create table: {}", e))),
        }
    }

    /// ## Drop table
    /// 
    /// Drop the specified database via gRPC call
    async fn drop_table(&self, request: Request<TableName>) -> Result<Response<Table>, Status> {
        let req_table = request.into_inner();
        let table = Table {
            name: req_table.name.clone(),
        };

        let mut db = DB.write().unwrap();
        let db = match &mut *db {
            Some(db) => db,
            None => return Err(Status::internal(String::from("Database is not available"))),
        };

        if let None = db.select_table(req_table.name.as_str()) {
            return Err(Status::not_found(String::from("Specified table does not exist")));
        }

        match db.drop_table(req_table.name.as_str()) {
            Ok(_) => return Ok(Response::new(table)),
            Err(e) => return Err(Status::internal(format!("Failed to drop table: {}", e))),
        }
    }

    /// ## List all table
    /// 
    /// This method list all available table via gRPC call
    async fn list_tables(&self, _: Request<Empty>) -> Result<Response<TableList>, Status> {
        let mut tables: Vec<String> = Vec::new();

        let db = DB.read().unwrap();
        let db = match &*db {
            Some(db) => db,
            None => return Err(Status::internal(String::from("Database is not available"))),
        };

        for table in db.get_tables() {
            tables.push(table.get_name().to_string());
        }

        let result = TableList {
            tables: tables,
        };
        return Ok(Response::new(result));
    }
}

pub async fn start_server(address: &String) {
    let hermes_grpc = HermesGrpc::default();

    Server::builder()
        .add_service(HermesServer::new(hermes_grpc))
        .serve(address.parse().unwrap())
        .await
        .unwrap();
}