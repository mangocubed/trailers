use async_graphql::{Context, Object, Result};

use crate::commands;
use crate::graphql::input_objects::UserInputObject;
use crate::graphql::objects::UserObject;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(
        &self,
        _ctx: &Context<'_>,
        input: UserInputObject,
    ) -> Result<UserObject<'_>> {
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
}
