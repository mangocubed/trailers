use async_graphql::{Context, EmptySubscription, ID, Schema, SchemaBuilder};

mod guards;
mod input_objects;
mod mutation_root;
mod objects;
mod query_root;

use mutation_root::MutationRoot;
use query_root::QueryRoot;
use uuid::Uuid;

use crate::models::User;

pub type GraphqlSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub trait GraphqlSchemaExt {
    fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription>;
}

impl GraphqlSchemaExt for GraphqlSchema {
    fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
        Schema::build(QueryRoot, MutationRoot, EmptySubscription)
    }
}

trait CustomContext {
    fn user(&self) -> &User;

    fn user_opt(&self) -> Option<&User>;
}

impl CustomContext for Context<'_> {
    fn user(&self) -> &User {
        self.data_unchecked::<User>()
    }

    fn user_opt(&self) -> Option<&User> {
        self.data_opt::<User>()
    }
}

trait IDExt {
    fn try_into_uuid(&self) -> Result<Uuid, uuid::Error>;
}

impl IDExt for ID {
    fn try_into_uuid(&self) -> Result<Uuid, uuid::Error> {
        Uuid::try_parse(self.as_ref())
    }
}
