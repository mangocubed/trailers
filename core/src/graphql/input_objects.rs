use async_graphql::InputObject;
use uuid::Uuid;

#[derive(InputObject)]
pub struct UserTitleTieInputObject {
    pub title_id: Uuid,
    #[graphql(default = true)]
    pub is_checked: bool,
    #[graphql(name = "videoId", deprecation = "Not used anymore")]
    pub video_id: Option<Uuid>,
}
