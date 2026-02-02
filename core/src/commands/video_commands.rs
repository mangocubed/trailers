use std::process::Command;

use chrono::{DateTime, Utc};

use crate::config::YT_DLP_CONFIG;
use crate::db_pool;
use crate::enums::{VideoOrientation, VideoSource, VideoType};
use crate::models::{Title, Video};

async fn delete_video(video: &Video<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM videos WHERE id = $1", video.id)
        .execute(db_pool)
        .await?;

    let _ = std::fs::remove_file(video.path());

    Ok(())
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
        let aspect_ratio = stdout_lines.next().and_then(|ar| ar.parse::<f32>().ok()).unwrap();
        let duration_secs = stdout_lines.next().and_then(|dur| dur.parse::<u32>().ok()).unwrap();

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
