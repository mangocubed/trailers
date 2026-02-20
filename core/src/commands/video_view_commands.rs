use crate::db_pool;
use crate::models::{User, Video};

pub async fn insert_video_view(video: &Video<'_>, user: &User) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "INSERT INTO video_views (video_id, user_id) VALUES ($1, $2)",
        video.id, // $1
        user.id,  // $2
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
