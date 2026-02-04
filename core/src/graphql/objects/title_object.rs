use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object};
use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use url::Url;
use uuid::Uuid;

use crate::commands;
use crate::enums::{TitleCrewJob, TitleMediaType};
use crate::graphql::CustomContext;
use crate::models::Title;
use crate::pagination::CursorParams;

use super::*;

pub struct TitleObject<'a>(pub Title<'a>);

#[Object]
impl TitleObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn media_type(&self) -> TitleMediaType {
        self.0.media_type
    }

    async fn backdrop_image_url(&self) -> Option<Url> {
        self.0.backdrop_image_url()
    }

    async fn poster_image_url(&self) -> Option<Url> {
        self.0.poster_image_url()
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn overview(&self) -> &str {
        &self.0.overview
    }

    async fn runtime(&self) -> Option<TimeDelta> {
        self.0
            .runtime
            .clone()
            .map(|value| TimeDelta::microseconds(value.microseconds))
    }

    async fn cast(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> async_graphql::Result<Connection<Uuid, TitleCastObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_title_cast(&CursorParams { after, first }, &self.0).await;

                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|title_cast| Edge::new(title_cast.id, TitleCastObject(title_cast))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn crew(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
        jobs: Option<Vec<TitleCrewJob>>,
    ) -> async_graphql::Result<Connection<Uuid, TitleCrewObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_title_crew(&CursorParams { after, first }, &self.0, jobs).await;

                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|title_crew| Edge::new(title_crew.id, TitleCrewObject(title_crew))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn genres(
        &self,
        after: Option<ID>,
        first: Option<i32>,
    ) -> async_graphql::Result<Connection<Uuid, GenreObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_genres(&CursorParams { after, first }, Some(&self.0)).await;

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

    async fn keywords(
        &self,
        after: Option<ID>,
        first: Option<i32>,
    ) -> async_graphql::Result<Connection<Uuid, KeywordObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_keywords(&CursorParams { after, first }, Some(&self.0)).await;

                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|keyword| Edge::new(keyword.id, KeywordObject(keyword))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn videos(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> async_graphql::Result<Connection<Uuid, VideoObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page =
                    commands::paginate_videos(CursorParams { after, first }, None, Some(&self.0), Some(true), None)
                        .await;

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

    async fn watch_providers(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
        country_code: Option<String>,
    ) -> async_graphql::Result<Connection<Uuid, TitleWatchProviderObject, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let page =
                    commands::paginate_title_watch_providers(&CursorParams { after, first }, &self.0, country_code)
                        .await;

                let mut connection = Connection::new(false, page.has_next_page);

                connection
                    .edges
                    .extend(page.nodes.into_iter().map(|title_watch_provider| {
                        Edge::new(title_watch_provider.id, TitleWatchProviderObject(title_watch_provider))
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn current_user_tie(&self, ctx: &Context<'_>) -> Option<UserTitleTieObject> {
        let user = ctx.user_opt()?;

        commands::get_user_title_tie(&user, &self.0)
            .await
            .map(UserTitleTieObject)
            .ok()
    }

    async fn released_on(&self) -> Option<NaiveDate> {
        self.0.released_on
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
