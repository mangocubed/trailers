use chrono::Utc;

use crate::models::{Title, User, UserTitleTie, Video};
use crate::{db_pool, jobs_storage};

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
    is_checked: bool,
    video: Option<&Video<'_>>,
) -> sqlx::Result<UserTitleTie> {
    let db_pool = db_pool().await;
    let (bookmarked_at, bookmarked_video_id) = if is_checked {
        (Some(Utc::now()), video.map(|v| v.id))
    } else {
        (None, None)
    };

    let user_title_tie = sqlx::query_as!(
        UserTitleTie,
        "UPDATE user_title_ties SET bookmarked_at = $2, bookmarked_video_id = $3 WHERE id = $1 RETURNING *",
        user_title_tie.id,   // $1
        bookmarked_at,       // $2
        bookmarked_video_id, // $3
    )
    .fetch_one(db_pool)
    .await?;

    if let Ok(user) = user_title_tie.user().await {
        jobs_storage().await.push_video_recommendations(&user).await;
    }

    Ok(user_title_tie)
}

pub async fn update_user_title_tie_like(
    user_title_tie: &UserTitleTie,
    is_checked: bool,
    video: Option<&Video<'_>>,
) -> sqlx::Result<UserTitleTie> {
    let db_pool = db_pool().await;
    let (liked_at, liked_video_id) = if is_checked {
        (Some(Utc::now()), video.map(|v| v.id))
    } else {
        (None, None)
    };

    let user_title_tie = sqlx::query_as!(
        UserTitleTie,
        "UPDATE user_title_ties SET liked_at = $2, liked_video_id = $3 WHERE id = $1 RETURNING *",
        user_title_tie.id, // $1
        liked_at,          // $2
        liked_video_id,    // $3
    )
    .fetch_one(db_pool)
    .await?;

    if let Ok(user) = user_title_tie.user().await {
        jobs_storage().await.push_video_recommendations(&user).await;
    }

    Ok(user_title_tie)
}

pub async fn update_user_title_tie_watched(
    user_title_tie: &UserTitleTie,
    is_checked: bool,
    video: Option<&Video<'_>>,
) -> sqlx::Result<UserTitleTie> {
    let db_pool = db_pool().await;
    let (watched_at, watched_video_id) = if is_checked {
        (Some(Utc::now()), video.map(|v| v.id))
    } else {
        (None, None)
    };

    let user_title_tie = sqlx::query_as!(
        UserTitleTie,
        "UPDATE user_title_ties SET watched_at = $2, watched_video_id = $3 WHERE id = $1 RETURNING *",
        user_title_tie.id, // $1
        watched_at,        // $2
        watched_video_id,  // $3
    )
    .fetch_one(db_pool)
    .await?;

    if let Ok(user) = user_title_tie.user().await {
        jobs_storage().await.push_video_recommendations(&user).await;
    }

    Ok(user_title_tie)
}
