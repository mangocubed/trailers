use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{ID, Object, Result};
use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::commands;
use crate::identity::IdentityUser;
use crate::models::User;
use crate::pagination::CursorParams;

use super::UserTitleTieObject;

pub struct IdentityUserObject<'a>(pub IdentityUser<'a>);

#[Object]
impl IdentityUserObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn username(&self) -> &str {
        &self.0.username
    }

    async fn display_name(&self) -> &str {
        &self.0.display_name
    }

    async fn initials(&self) -> &str {
        &self.0.initials
    }

    async fn language_code(&self) -> &str {
        &self.0.language_code
    }

    async fn country_code(&self) -> &str {
        &self.0.country_code
    }

    async fn avatar_image_url(&self) -> Url {
        self.0.avatar_image_url.clone()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct UserObject(pub User);

#[Object]
impl UserObject {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn identity_user(&self) -> Result<IdentityUserObject<'_>> {
        Ok(self.0.identity_user().await.map(IdentityUserObject)?)
    }

    async fn title_ties(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
        is_bookmarked: Option<bool>,
        is_watched: Option<bool>,
    ) -> Result<Connection<Uuid, UserTitleTieObject, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_user_title_ties(
                    &CursorParams { after, first },
                    &self.0,
                    is_bookmarked,
                    is_watched,
                )
                .await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|user_title_tie| Edge::new(user_title_tie.id, UserTitleTieObject(user_title_tie))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
