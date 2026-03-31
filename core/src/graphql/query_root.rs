use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object};
use uuid::Uuid;

use crate::enums::TitleMediaType;
use crate::graphql::objects::{GenreObject, InfoObject, TitleObject, UserObject, WatchProviderObject};
use crate::graphql::{CustomContext, IDExt};
use crate::pagination::CursorParams;
use crate::{Info, commands};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn current_user(&self, ctx: &Context<'_>) -> Option<UserObject> {
        ctx.user_opt().map(|user| UserObject(user.clone()))
    }

    async fn genres(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
        ids: Option<Vec<Uuid>>,
    ) -> async_graphql::Result<Connection<Uuid, GenreObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_genres(&CursorParams { after, first }, ids, None).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|genre| Edge::new(genre.id, GenreObject(genre))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }

    async fn title(&self, id: ID) -> Option<TitleObject<'_>> {
        commands::get_title_by_id(id.try_into_uuid().ok()?, None, None)
            .await
            .map(TitleObject)
            .ok()
    }

    async fn titles(
        &self,
        ctx: &Context<'_>,
        after: Option<Uuid>,
        first: Option<i32>,
        #[graphql(name = "query")] q: Option<String>,
        media_type: Option<TitleMediaType>,
        genre_ids: Option<Vec<Uuid>>,
        watch_provider_ids: Option<Vec<Uuid>>,
        country_code: Option<String>,
        include_viewed: Option<bool>,
        include_without_videos: Option<bool>,
    ) -> async_graphql::Result<Connection<Uuid, TitleObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let user = ctx.user_opt();
                let cursor_page = commands::paginate_titles(
                    CursorParams { after, first },
                    user,
                    q.as_deref(),
                    media_type,
                    genre_ids,
                    watch_provider_ids,
                    country_code.as_deref(),
                    include_viewed,
                    include_without_videos,
                )
                .await;
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

    async fn watch_providers(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
        ids: Option<Vec<Uuid>>,
        country_code: Option<String>,
    ) -> async_graphql::Result<Connection<Uuid, WatchProviderObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let page =
                    commands::paginate_watch_providers(CursorParams { after, first }, ids, country_code.as_deref())
                        .await;

                let mut connection = Connection::new(false, page.has_next_page);

                connection.edges.extend(
                    page.nodes
                        .into_iter()
                        .map(|watch_provider| Edge::new(watch_provider.id, WatchProviderObject(watch_provider))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
