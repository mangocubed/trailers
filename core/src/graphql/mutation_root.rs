use async_graphql::{Context, Object, Result};

use crate::commands;
use crate::graphql::CustomContext;
use crate::graphql::guards::{GuestGuard, UserGuard};
use crate::graphql::input_objects::{SessionInputObject, UserInputObject};
use crate::graphql::objects::{SessionObject, UserObject};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    #[graphql(guard = "GuestGuard::default()")]
    async fn create_user(&self, _ctx: &Context<'_>, input: UserInputObject) -> Result<UserObject<'_>> {
        let user = commands::insert_user(
            &input.username,
            &input.email,
            &input.password,
            &input.full_name,
            input.birthdate,
            &input.country_code,
        )
        .await?;

        Ok(UserObject(user))
    }

    #[graphql(guard = "GuestGuard::default()")]
    async fn create_session(&self, ctx: &Context<'_>, input: SessionInputObject) -> Result<SessionObject<'_>> {
        let user = commands::authenticate_user(&input.username_or_email, &input.password).await?;

        let client_ip = ctx.client_ip();

        let session = commands::insert_session(&user, *client_ip).await?;

        Ok(SessionObject(session))
    }

    #[graphql(guard = "UserGuard::default()")]
    async fn finish_session(&self, ctx: &Context<'_>) -> Result<bool> {
        let session = ctx.session();

        commands::finish_session(&session).await?;

        Ok(true)
    }
}
