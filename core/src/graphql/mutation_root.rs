use async_graphql::{Context, Object, Result};

use crate::commands;
use crate::graphql::CustomContext;
use crate::graphql::guards::{GuestGuard, UserGuard};
use crate::graphql::input_objects::{SessionInputObject, UserInputObject, UserTitleTieInputObject};
use crate::graphql::objects::{SessionObject, UserObject, UserTitleTieObject};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    #[graphql(guard = "GuestGuard")]
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

    #[graphql(guard = "GuestGuard")]
    async fn create_session(&self, ctx: &Context<'_>, input: SessionInputObject) -> Result<SessionObject<'_>> {
        let user = commands::authenticate_user(&input.username_or_email, &input.password).await?;

        let client_ip = ctx.client_ip();

        let session = commands::insert_session(&user, *client_ip).await?;

        Ok(SessionObject(session))
    }

    #[graphql(guard = "UserGuard")]
    async fn finish_session(&self, ctx: &Context<'_>) -> Result<bool> {
        let session = ctx.session();

        commands::finish_session(session).await?;

        Ok(true)
    }

    #[graphql(guard = "UserGuard")]
    async fn update_bookmark(&self, ctx: &Context<'_>, input: UserTitleTieInputObject) -> Result<UserTitleTieObject> {
        let user = ctx.user();
        let title = commands::get_title_by_id(input.title_id, None).await?;
        let user_title_tie = commands::get_or_insert_user_title_tie(user, &title).await?;
        let video = if let Some(video_id) = input.video_id {
            Some(commands::get_video_by_id(video_id, None).await?)
        } else {
            None
        };

        Ok(
            commands::update_user_title_tie_bookmark(&user_title_tie, input.is_checked, video.as_ref())
                .await
                .map(UserTitleTieObject)?,
        )
    }

    #[graphql(guard = "UserGuard")]
    async fn update_like(&self, ctx: &Context<'_>, input: UserTitleTieInputObject) -> Result<UserTitleTieObject> {
        let user = ctx.user();
        let title = commands::get_title_by_id(input.title_id, None).await?;
        let user_title_tie = commands::get_or_insert_user_title_tie(user, &title).await?;
        let video = if let Some(video_id) = input.video_id {
            Some(commands::get_video_by_id(video_id, None).await?)
        } else {
            None
        };

        Ok(
            commands::update_user_title_tie_like(&user_title_tie, input.is_checked, video.as_ref())
                .await
                .map(UserTitleTieObject)?,
        )
    }

    #[graphql(guard = "UserGuard")]
    async fn update_watched(&self, ctx: &Context<'_>, input: UserTitleTieInputObject) -> Result<UserTitleTieObject> {
        let user = ctx.user();
        let title = commands::get_title_by_id(input.title_id, None).await?;
        let user_title_tie = commands::get_or_insert_user_title_tie(user, &title).await?;
        let video = if let Some(video_id) = input.video_id {
            Some(commands::get_video_by_id(video_id, None).await?)
        } else {
            None
        };

        Ok(
            commands::update_user_title_tie_watched(&user_title_tie, input.is_checked, video.as_ref())
                .await
                .map(UserTitleTieObject)?,
        )
    }
}
