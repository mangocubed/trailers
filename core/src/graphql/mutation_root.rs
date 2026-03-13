use async_graphql::{Context, Object, Result};

use crate::commands;
use crate::graphql::CustomContext;
use crate::graphql::guards::UserGuard;
use crate::graphql::input_objects::UserTitleTieInputObject;
use crate::graphql::objects::UserTitleTieObject;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    #[graphql(guard = "UserGuard")]
    async fn create_user_title_tie(
        &self,
        ctx: &Context<'_>,
        input: UserTitleTieInputObject,
    ) -> Result<UserTitleTieObject> {
        let user = ctx.user();
        let title = commands::get_title_by_id(input.title_id, None, None).await?;

        Ok(commands::get_or_insert_user_title_tie(user, &title)
            .await
            .map(UserTitleTieObject)?)
    }

    #[graphql(guard = "UserGuard")]
    async fn update_bookmark(&self, ctx: &Context<'_>, input: UserTitleTieInputObject) -> Result<UserTitleTieObject> {
        let user = ctx.user();
        let title = commands::get_title_by_id(input.title_id, None, None).await?;
        let user_title_tie = commands::get_or_insert_user_title_tie(user, &title).await?;

        Ok(
            commands::update_user_title_tie_bookmark(&user_title_tie, input.is_checked)
                .await
                .map(UserTitleTieObject)?,
        )
    }

    #[graphql(guard = "UserGuard")]
    async fn update_like(&self, ctx: &Context<'_>, input: UserTitleTieInputObject) -> Result<UserTitleTieObject> {
        let user = ctx.user();
        let title = commands::get_title_by_id(input.title_id, None, None).await?;
        let user_title_tie = commands::get_or_insert_user_title_tie(user, &title).await?;

        Ok(commands::update_user_title_tie_like(&user_title_tie, input.is_checked)
            .await
            .map(UserTitleTieObject)?)
    }

    #[graphql(guard = "UserGuard")]
    async fn update_watched(&self, ctx: &Context<'_>, input: UserTitleTieInputObject) -> Result<UserTitleTieObject> {
        let user = ctx.user();
        let title = commands::get_title_by_id(input.title_id, None, None).await?;
        let user_title_tie = commands::get_or_insert_user_title_tie(user, &title).await?;

        Ok(
            commands::update_user_title_tie_watched(&user_title_tie, input.is_checked)
                .await
                .map(UserTitleTieObject)?,
        )
    }
}
