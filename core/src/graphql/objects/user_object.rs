use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use toolbox::graphql::objects::IdentityUserObject;
use toolbox::pagination::CursorParams;

use crate::commands;
use crate::graphql::CustomContext;
use crate::models::User;

use super::UserTitleTieObject;

pub struct UserObject(pub User);

#[Object]
impl UserObject {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn identity_user(&self, ctx: &Context<'_>) -> Result<IdentityUserObject<'_>> {
        Ok(self
            .0
            .identity_user(ctx.identity_client())
            .await
            .map(IdentityUserObject)?)
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
