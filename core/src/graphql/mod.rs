use async_graphql::{Context, EmptySubscription, ID, Result, Schema, SchemaBuilder};
use uuid::Uuid;

mod guards;
mod input_objects;
mod mutation_root;
mod objects;
mod query_root;

use toolbox::identity_client::IdentityClient;

use crate::models::User;

use mutation_root::MutationRoot;
use query_root::QueryRoot;

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
    fn identity_client(&self) -> &IdentityClient;

    fn user(&self) -> &User;

    fn user_opt(&self) -> Option<&User>;
}

impl CustomContext for Context<'_> {
    fn identity_client(&self) -> &IdentityClient {
        self.data_unchecked::<IdentityClient>()
    }

    fn user(&self) -> &User {
        self.data_unchecked::<User>()
    }

    fn user_opt(&self) -> Option<&User> {
        self.data_opt::<User>()
    }
}

trait IDExt {
    fn try_into_uuid(&self) -> Result<Uuid>;
}

impl IDExt for ID {
    fn try_into_uuid(&self) -> Result<Uuid> {
        Ok(Uuid::try_parse(self.as_ref())?)
    }
}
