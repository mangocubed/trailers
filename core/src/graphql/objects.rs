use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{ID, Object, Result};
use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use url::Url;
use uuid::Uuid;

use crate::enums::TitleMediaType;
use crate::models::{Genre, Session, Title, User};
use crate::pagination::CursorParams;
use crate::{Info, commands};

pub struct InfoObject(pub Info);

#[Object]
impl InfoObject {
    async fn built_at(&self) -> DateTime<Utc> {
        self.0.built_at
    }

    async fn version(&self) -> &str {
        &self.0.version
    }
}

pub struct GenreObject<'a>(pub Genre<'a>);

#[Object]
impl GenreObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct SessionObject<'a>(pub Session<'a>);

#[Object]
impl SessionObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn user_id(&self) -> ID {
        self.0.user_id.into()
    }

    async fn user(&self) -> Result<UserObject<'_>> {
        Ok(UserObject(self.0.user().await?))
    }

    async fn token(&self) -> &str {
        &self.0.token
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct TitleObject<'a>(pub Title<'a>);

#[Object]
impl TitleObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn media_type(&self) -> TitleMediaType {
        self.0.media_type
    }

    async fn tmdb_backdrop_image_url(&self) -> Option<Url> {
        self.0.backdrop_url()
    }

    async fn tmdb_poster_image_url(&self) -> Option<Url> {
        self.0.poster_url()
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
                let page = commands::paginate_genres(&CursorParams { after, first }, Some(&self.0)).await;

                let mut connection = Connection::new(false, page.has_next_page);

                connection.edges.extend(
                    page.nodes
                        .into_iter()
                        .map(|title_genre| Edge::new(title_genre.id, GenreObject(title_genre))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
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

pub struct UserObject<'a>(pub User<'a>);

#[Object]
impl UserObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn username(&self) -> &str {
        &self.0.username
    }

    async fn display_name(&self) -> &str {
        &self.0.display_name
    }

    async fn full_name(&self) -> &str {
        &self.0.full_name
    }

    async fn initials(&self) -> String {
        self.0.initials()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
