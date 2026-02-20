use async_graphql::InputObject;
use uuid::Uuid;

#[derive(InputObject)]
pub struct UserTitleTieInputObject {
    pub title_id: Uuid,
    pub is_checked: bool,
    pub video_id: Option<Uuid>,
}
