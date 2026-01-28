use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder};
use chrono::{DateTime, Utc};

use crate::Info;

pub struct InfoObject(Info);

#[Object]
impl InfoObject {
    async fn built_at(&self) -> DateTime<Utc> {
        self.0.built_at
    }

    async fn version(&self) -> &str {
        &self.0.version
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }
}

pub type GraphqlSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub trait GraphqlSchemaExt {
    fn builder() -> SchemaBuilder<QueryRoot, EmptyMutation, EmptySubscription>;
}

impl GraphqlSchemaExt for GraphqlSchema {
    fn builder() -> SchemaBuilder<QueryRoot, EmptyMutation, EmptySubscription> {
        Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
    }
}
