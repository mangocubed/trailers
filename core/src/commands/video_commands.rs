use std::process::Command;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::config::YT_DLP_CONFIG;
use crate::enums::{VideoOrientation, VideoSource, VideoType};
use crate::models::{Title, Video};
use crate::pagination::{CursorPage, CursorParams};
use crate::{db_pool, jobs_storage};

async fn delete_video(video: &Video<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM videos WHERE id = $1", video.id)
        .execute(db_pool)
        .await?;

    let _ = std::fs::remove_file(video.path());

    Ok(())
}

pub fn generate_video_hls(video: &Video<'_>) -> anyhow::Result<()> {
    let hls_path = video.hls_path();

    if std::fs::exists(&hls_path).unwrap_or_default() {
        return Ok(());
    }

    let hls_dir = hls_path.parent().expect("Could not get parent directory");

    std::fs::create_dir_all(hls_dir)?;

    let _ = Command::new("ffmpeg")
        .args([
            "-i",
            video.path().to_str().unwrap(),
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-crf",
            "28",
            "-preset",
            "slower",
            "-f",
            "hls",
            "-hls_time",
            "6",
            "-hls_playlist_type",
            "vod",
            "-hls_segment_filename",
            hls_dir.join("segment-%02d.ts").to_str().unwrap(),
            hls_path.to_str().unwrap(),
        ])
        .output()?;

    Ok(())
}

pub async fn get_video_by_id<'a>(id: Uuid) -> sqlx::Result<Video<'a>> {
    let db_pool = db_pool().await;

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
            published_at,
            created_at,
            updated_at
        FROM videos WHERE downloaded_at IS NOT NULL AND id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_video(
    title: &Title<'_>,
    tmdb_id: &str,
    source: VideoSource,
    source_key: &str,
    name: &str,
    video_type: VideoType,
    language: &str,
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
        && std::fs::exists(&output_path).unwrap_or_default()
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

        jobs_storage().await.push_generate_video_hls(&video).await;

        let orientation = VideoOrientation::from_aspect_ratio(aspect_ratio);

        let _ = update_video_info(&video, duration_secs, orientation, Utc::now()).await;

        Ok(())
    } else {
        delete_video(&video).await?;

        Err(anyhow::anyhow!("Could not download video"))
    }
}

pub async fn paginate_videos<'a>(cursor_params: CursorParams, title: &Title<'_>) -> CursorPage<Video<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Video| node.id,
        async |after| get_video_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_duration_secs, cursor_published_at) = cursor_resource
                .map(|c| (Some(c.id), Some(c.duration_secs), Some(c.published_at)))
                .unwrap_or_default();

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
                    published_at,
                    created_at,
                    updated_at
                FROM videos
                WHERE downloaded_at IS NOT NULL
                    AND (
                        $1::uuid IS NULL
                        OR (duration_secs < $2) OR (duration_secs = $2 AND published_at < $3)
                        OR (published_at = $3 AND id < $1)
                    ) AND title_id = $4
                ORDER BY orientation::text DESC, duration_secs DESC, published_at DESC, id DESC LIMIT $5"#,
                cursor_id,            // $1
                cursor_duration_secs, // $2
                cursor_published_at,  // $3
                title.id,             // $4
                limit                 // $5
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
    downloaded_at: DateTime<Utc>,
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

    sqlx::query!(
        "UPDATE titles SET has_videos = TRUE WHERE id = $1 AND has_videos IS FALSE",
        video.title_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
