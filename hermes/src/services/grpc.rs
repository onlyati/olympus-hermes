#![allow(dead_code)]

use tonic::{transport::Server, Request, Response, Status};

use hermes::hermes_server::{Hermes, HermesServer};
use hermes::{SetPair, Key, TableName, Empty, ActionResult, Pair, KeyList, Table, TableList};

use crate::DB;

mod hermes {
    tonic::include_proto!("hermes");
}

#[derive(Debug, Default)]
struct HermesGrpc {}

#[tonic::async_trait]
impl Hermes for HermesGrpc {
    async fn set(&self, request: Request<SetPair>) -> Result<Response<Pair>, Status> {
        unimplemented!()
    }

    async fn get(&self, request: Request<Key>) -> Result<Response<Pair>, Status> {
        unimplemented!()
    }

    async fn delete(&self, request: Request<Key>) -> Result<Response<Pair>, Status> {
        unimplemented!()
    }

    async fn mask(&self, request: Request<Key>) -> Result<Response<KeyList>, Status> {
        unimplemented!()
    }

    async fn create_table(&self, request: Request<TableName>) -> Result<Response<Table>, Status> {
        unimplemented!()
    }

    async fn drop_table(&self, request: Request<TableName>) -> Result<Response<Table>, Status> {
        unimplemented!()
    }

    async fn list_tables(&self, request: Request<Empty>) -> Result<Response<TableList>, Status> {
        unimplemented!()
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