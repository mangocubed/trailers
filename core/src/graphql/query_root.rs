use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object};
use uuid::Uuid;

use crate::graphql::objects::{InfoObject, TitleObject, UserObject, VideoObject};
use crate::graphql::{CustomContext, IDExt};
use crate::pagination::CursorParams;
use crate::{Info, commands};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn current_user<'a>(&self, ctx: &'a Context<'_>) -> Option<UserObject> {
        ctx.user_opt().map(|user| UserObject(user.clone()))
    }

    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }

    async fn title(&self, id: ID) -> Option<TitleObject<'_>> {
        commands::get_title_by_id(id.try_into_uuid().ok()?, None)
            .await
            .map(TitleObject)
            .ok()
    }

    async fn titles(
        &self,
        #[graphql(name = "query")] q: Option<String>,
        after: Option<Uuid>,
        first: Option<i32>,
        has_videos: Option<bool>,
    ) -> async_graphql::Result<Connection<Uuid, TitleObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_titles(CursorParams { after, first }, q, has_videos).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|title| Edge::new(title.id, TitleObject(title))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn user(&self, username: String) -> Option<UserObject> {
        let identity_user = commands::get_identity_user(&username).await.ok()?;

        commands::get_user_by_identity_user(&identity_user)
            .await
            .map(UserObject)
            .ok()
    }

    async fn video(&self, ctx: &Context<'_>, id: ID) -> Option<VideoObject<'_>> {
        let user = ctx.user_opt();
        let video = commands::get_video_by_id(id.try_into_uuid().ok()?, user).await.ok()?;

        if let Some(user) = user {
            let _ = commands::insert_video_view(&video, user).await;
        }

        Some(VideoObject(video))
    }

    async fn videos<'a>(
        &self,
        ctx: &'a Context<'_>,
        after: Option<Uuid>,
        first: Option<i32>,
        include_viewed: Option<bool>,
    ) -> async_graphql::Result<Connection<Uuid, VideoObject<'a>, EmptyFields, EmptyFields>> {
        query(
            after.map(|id| id.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let user = ctx.user_opt();
                let cursor_page =
                    commands::paginate_videos(CursorParams { after, first }, user, None, include_viewed, None).await;

                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|video| Edge::new(video.id, VideoObject(video))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
