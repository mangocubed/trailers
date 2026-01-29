use async_graphql::Object;

use crate::Info;
use crate::graphql::objects::InfoObject;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }
}
