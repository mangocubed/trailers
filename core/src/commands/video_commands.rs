use std::process::Command;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::config::YT_DLP_CONFIG;
use crate::db_pool;
use crate::enums::{VideoOrientation, VideoSource, VideoType};
use crate::models::{Title, User, Video};
use crate::pagination::{CursorPage, CursorParams};

async fn delete_video(video: &Video<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM videos WHERE id = $1", video.id)
        .execute(db_pool)
        .await?;

    let _ = std::fs::remove_file(video.path());

    Ok(())
}

pub async fn get_video_by_id<'a>(id: Uuid, user: Option<&User<'_>>) -> sqlx::Result<Video<'a>> {
    let db_pool = db_pool().await;

    let user_id = user.map(|u| u.id);

    sqlx::query_as!(
        Video,
        r#"SELECT
            id,
            title_id,
            tmdb_id,
            source as "source!: VideoSource",
            source_key,
            name,
            video_type as "video_type!: VideoType",
            duration_secs,
            orientation as "orientation!: VideoOrientation",
            language,
            CASE WHEN $2::uuid IS NULL THEN 0 ELSE get_title_relevance(title_id, $2) END as "relevance!",
            published_at,
            created_at,
            updated_at
        FROM videos WHERE downloaded_at IS NOT NULL AND id = $1 LIMIT 1"#,
        id,      // $1
        user_id  // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_video<'a>(
    title: &'a Title<'_>,
    tmdb_id: &'a str,
    source: VideoSource,
    source_key: &'a str,
    name: &'a str,
    video_type: VideoType,
    language: &'a str,
    published_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    let db_pool = db_pool().await;

    let video = sqlx::query_as!(
        Video,
        r#"INSERT INTO videos (title_id, tmdb_id, source, source_key, name, video_type, language, published_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            title_id,
            tmdb_id,
            source as "source!: VideoSource",
            source_key,
            name,
            video_type as "video_type!: VideoType",
            duration_secs,
            orientation as "orientation!: VideoOrientation",
            language,
            0 AS "relevance!",
            published_at,
            created_at,
            updated_at"#,
        title.id,                // $1
        tmdb_id,                 // $2
        source as VideoSource,   // $3
        source_key,              // $4
        name,                    // $5
        video_type as VideoType, // $6
        language,                // $7
        published_at,            // $8
    )
    .fetch_one(db_pool)
    .await?;

    let output_path = video.path();

    let mut args = vec![
        "--format",
        "bestvideo[width<=1920][height<=1920][filesize<=100M][ext=mp4]+bestaudio[ext=m4a]/best[width<=1920][height<=1920][filesize<=100M][ext=mp4]",
        "--max-filesize",
        "100M",
        "--print",
        "aspect_ratio,duration",
        "--no-simulate",
        "--quiet",
        "--output",
        output_path.to_str().unwrap(),
    ];

    if let Some(proxy) = &YT_DLP_CONFIG.proxy {
        args.append(&mut vec!["--proxy", proxy]);
    }

    args.push(source_key);

    let result = Command::new("yt-dlp").args(args).output();

    if let Ok(ref output) = result
        && output.status.success()
        && let Ok(stdout) = String::from_utf8(output.stdout.clone())
    {
        let mut stdout_lines = stdout.lines();

        let Some(Ok(aspect_ratio)) = stdout_lines.next().map(|ar| ar.parse::<f32>()) else {
            delete_video(&video).await?;

            return Err(anyhow::anyhow!("Failed to parse aspect ratio"));
        };

        let Some(Ok(duration_secs)) = stdout_lines.next().map(|dur| dur.parse::<u32>()) else {
            delete_video(&video).await?;

            return Err(anyhow::anyhow!("Failed to parse duration"));
        };

        if duration_secs > 600 {
            delete_video(&video).await?;

            return Err(anyhow::anyhow!("Video duration exceeds maximum allowed"));
        }

        let orientation = VideoOrientation::from_aspect_ratio(aspect_ratio);

        let _ = update_video_info(&video, duration_secs, orientation, Some(Utc::now())).await;

        Ok(())
    } else {
        delete_video(&video).await?;

        Err(anyhow::anyhow!("Could not download video"))
    }
}

pub async fn paginate_videos<'a>(
    cursor_params: CursorParams,
    user: Option<&'a User<'_>>,
    title: Option<&'a Title<'_>>,
    include_viewed: Option<bool>,
    include_adult: Option<bool>,
) -> CursorPage<Video<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Video| node.id,
        async |after| get_video_by_id(after, user).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_relevance, cursor_published_at) = cursor_resource
                .map(|c| (Some(c.id), Some(c.relevance), Some(c.published_at)))
                .unwrap_or_default();
            let user_id = user.map(|u| u.id);
            let title_id = title.map(|t| t.id);

            sqlx::query_as!(
                Video,
                r#"SELECT
                id as "id!",
                title_id as "title_id!",
                tmdb_id as "tmdb_id!",
                "source!: VideoSource",
                source_key as "source_key!",
                name as "name!",
                "video_type!: VideoType",
                duration_secs as "duration_secs!",
                "orientation!: VideoOrientation",
                language as "language!",
                relevance as "relevance!",
                published_at as "published_at!",
                created_at as "created_at!",
                updated_at
            FROM ((
                SELECT
                    v1.id,
                    title_id,
                    tmdb_id,
                    source as "source!: VideoSource",
                    source_key,
                    name,
                    video_type as "video_type!: VideoType",
                    duration_secs,
                    orientation as "orientation!: VideoOrientation",
                    language,
                    vr.relevance,
                    published_at,
                    v1.created_at,
                    v1.updated_at
                FROM videos AS v1, video_recommendations AS vr
                WHERE
                    v1.downloaded_at IS NOT NULL AND vr.user_id = $4
                    AND v1.id = vr.video_id AND (
                        $1::uuid IS NULL OR $2::bigint IS NULL OR $3::timestamptz IS NULL
                        OR relevance < $2 OR (relevance = $2 AND published_at < $3)
                        OR (published_at = $3 AND v1.id < $1)
                    ) AND (
                        $6 IS TRUE OR (
                            SELECT vv.id FROM video_views AS vv
                            WHERE
                                vv.user_id = $4 AND (
                                    vv.video_id = v1.id OR (
                                        vv.created_at > current_timestamp - INTERVAL '1 hour'
                                        AND (
                                            SELECT v2.id FROM videos AS v2
                                            WHERE v2.id = vv.video_id AND v2.title_id = v1.title_id
                                            LIMIT 1
                                        ) IS NOT NULL
                                    )
                                )
                            LIMIT 1
                        ) IS NULL
                    ) AND ($5::uuid IS NULL OR title_id = $5)
                    AND (
                        $7 IS TRUE OR (
                            SELECT is_adult FROM titles AS t WHERE t.id = v1.title_id LIMIT 1
                        ) IS FALSE
                    )
                ORDER BY relevance DESC, orientation::text DESC, published_at DESC, v1.id DESC LIMIT $8
            ) UNION ALL (
                SELECT
                    id,
                    title_id,
                    tmdb_id,
                    source as "source!: VideoSource",
                    source_key,
                    name,
                    video_type as "video_type!: VideoType",
                    duration_secs,
                    orientation as "orientation!: VideoOrientation",
                    language,
                    0 as "relevance!",
                    published_at,
                    created_at,
                    updated_at
                FROM videos AS v2
                WHERE downloaded_at IS NOT NULL AND (
                        $1::uuid IS NULL OR $3::timestamptz IS NULL OR published_at < $3
                        OR (published_at = $3 AND id < $1)
                    ) AND (
                        $4::uuid IS NULL OR $6 IS TRUE OR (
                            SELECT vv.id FROM video_views AS vv WHERE vv.video_id = v2.id AND vv.user_id = $4
                            LIMIT 1
                        ) IS NULL
                    ) AND ($5::uuid IS NULL OR title_id = $5)
                    AND (
                        $7 IS TRUE OR (
                            SELECT is_adult FROM titles AS t WHERE t.id = v2.title_id LIMIT 1
                        ) IS FALSE
                    )
                ORDER BY orientation::text DESC, published_at DESC, id DESC LIMIT $8
            )) AS sub LIMIT $8"#,
                cursor_id,           // $1
                cursor_relevance,    // $2
                cursor_published_at, // $3
                user_id,             // $4
                title_id,            // $5
                include_viewed,      // $6
                include_adult,       // $7
                limit                // $8
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

async fn update_video_info(
    video: &Video<'_>,
    duration_secs: u32,
    orientation: VideoOrientation,
    downloaded_at: Option<DateTime<Utc>>,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE videos SET duration_secs = $2, orientation = $3, downloaded_at = $4 WHERE id = $1",
        video.id,             // $
        duration_secs as i32, // $2
        orientation as _,     // $3
        downloaded_at,        // $4
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
