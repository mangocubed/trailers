use chrono::Utc;
use uuid::Uuid;

use crate::models::{Title, User, UserTitleTie, Video};
use crate::pagination::{CursorPage, CursorParams};
use crate::{db_pool, jobs_storage};

pub async fn get_or_insert_user_title_tie(user: &User<'_>, title: &Title<'_>) -> sqlx::Result<UserTitleTie> {
    if let Ok(user_title_tie) = get_user_title_tie(user, title).await {
        return Ok(user_title_tie);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        UserTitleTie,
        "INSERT INTO user_title_ties (user_id, title_id) VALUES ($1, $2) RETURNING *",
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

async fn get_user_title_tie_by_id(id: Uuid) -> sqlx::Result<UserTitleTie> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        UserTitleTie,
        "SELECT * FROM user_title_ties WHERE id = $1 LIMIT 1",
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn paginate_user_title_ties(
    cursor_params: &CursorParams,
    user: &User<'_>,
    is_bookmarked: Option<bool>,
    is_watched: Option<bool>,
) -> CursorPage<UserTitleTie> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &UserTitleTie| node.id,
        async |after| get_user_title_tie_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_created_at) = cursor_resource
                .map(|r| (Some(r.id), Some(r.created_at)))
                .unwrap_or_default();

            sqlx::query_as!(
                UserTitleTie,
                r#"SELECT * FROM user_title_ties
                    WHERE (
                            $1::uuid IS NULL OR $2::timestamptz IS NULL OR created_at < $2
                            OR (created_at = $2 AND id < $1)
                        ) AND ($3::uuid IS NULL OR user_id = $3)
                        AND (
                            $4::bool IS NULL OR ($4 IS TRUE AND bookmarked_at IS NOT NULL)
                            OR ($4 IS FALSE AND bookmarked_at IS NULL)
                        ) AND (
                            $5::bool IS NULL OR ($5 IS TRUE AND watched_at IS NOT NULL)
                            OR ($5 IS FALSE AND liked_at IS NULL)
                        )
                    ORDER BY created_at DESC, id DESC LIMIT $6"#,
                cursor_id,         // $1
                cursor_created_at, // $2
                user.id,           // $3
                is_bookmarked,     // $4
                is_watched,        // $5
                limit              // $6
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
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
