use std::net::IpAddr;

use async_graphql::{Context, EmptySubscription, ID, Schema, SchemaBuilder};

mod guards;
mod input_objects;
mod input_validators;
mod mutation_root;
mod objects;
mod query_root;

use mutation_root::MutationRoot;
use query_root::QueryRoot;
use uuid::Uuid;

use crate::models::{Session, User};

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
    fn client_ip(&self) -> &IpAddr;

    fn session(&self) -> &Session<'_>;

    fn session_opt(&self) -> Option<&Session<'_>>;

    fn user(&self) -> &User<'_>;

    fn user_opt(&self) -> Option<&User<'_>>;
}

impl CustomContext for Context<'_> {
    fn client_ip(&self) -> &IpAddr {
        self.data_unchecked::<IpAddr>()
    }

    fn session(&self) -> &Session<'_> {
        self.data_unchecked::<Session>()
    }

    fn session_opt(&self) -> Option<&Session<'_>> {
        self.data_opt::<Session>()
    }

    fn user(&self) -> &User<'_> {
        self.data_unchecked::<User>()
    }

    fn user_opt(&self) -> Option<&User<'_>> {
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
