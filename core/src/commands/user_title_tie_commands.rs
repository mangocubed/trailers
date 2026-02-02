use crate::db_pool;
use crate::models::{Title, User, UserTitleTie, Video};

pub async fn get_or_insert_user_title_tie(user: &User<'_>, title: &Title<'_>) -> sqlx::Result<UserTitleTie> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        UserTitleTie,
        "INSERT INTO user_title_ties (user_id, title_id) VALUES ($1, $2) ON CONFLICT (user_id, title_id) DO NOTHING RETURNING *",
        user.id,  // $1
        title.id, // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_user_title_tie(user: &User<'_>, title: &Title<'_>) -> sqlx::Result<UserTitleTie> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        UserTitleTie,
        "SELECT * FROM user_title_ties WHERE user_id = $1 AND title_id = $2 LIMIT 1",
        user.id,  // $1
        title.id, // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn update_user_title_tie_bookmark(
    user_title_tie: &UserTitleTie,
    is_active: bool,
    video: Option<&Video<'_>>,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;
    let video_id = if is_active { video.map(|v| v.id) } else { None };

    sqlx::query!(
        "UPDATE user_title_ties
        SET bookmarked_at = CASE WHEN $2 IS TRUE THEN current_timestamp ELSE NULL END, bookmarked_video_id = $3
        WHERE id = $1",
        user_title_tie.id, // $1
        is_active,         // $2
        video_id,          // $3
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn update_user_title_tie_like(
    user_title_tie: &UserTitleTie,
    is_active: bool,
    video: Option<&Video<'_>>,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;
    let video_id = if is_active { video.map(|v| v.id) } else { None };

    sqlx::query!(
        "UPDATE user_title_ties
        SET liked_at = CASE WHEN $2 IS TRUE THEN current_timestamp ELSE NULL END, liked_video_id = $3
        WHERE id = $1",
        user_title_tie.id, // $1
        is_active,         // $2
        video_id,          // $3
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn update_user_title_tie_watched(
    user_title_tie: &UserTitleTie,
    is_active: bool,
    video: Option<&Video<'_>>,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;
    let video_id = if is_active { video.map(|v| v.id) } else { None };

    sqlx::query!(
        "UPDATE user_title_ties
        SET watched_at = CASE WHEN $2 IS TRUE THEN current_timestamp ELSE NULL END, watched_video_id = $3
        WHERE id = $1",
        user_title_tie.id, // $1
        is_active,         // $2
        video_id,          // $3
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
