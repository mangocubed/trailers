use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

use crate::graphql::objects::{InfoObject, TitleObject, UserObject};
use crate::graphql::{CustomContext, IDExt};
use crate::pagination::CursorParams;
use crate::{Info, commands};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn current_user<'a>(&self, ctx: &'a Context<'_>) -> Option<UserObject<'a>> {
        ctx.user_opt().map(|user| UserObject(user.clone()))
    }

    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }

    async fn title(&self, id: ID) -> Result<TitleObject<'_>> {
        Ok(commands::get_title_by_id(id.try_into_uuid()?, None)
            .await
            .map(TitleObject)?)
    }

    async fn titles(
        &self,
        #[graphql(name = "query")] q: Option<String>,
        after: Option<ID>,
        first: Option<i32>,
    ) -> async_graphql::Result<Connection<Uuid, TitleObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_titles(q, &CursorParams { after, first }).await;
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
}
