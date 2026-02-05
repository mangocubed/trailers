use url::Url;
use uuid::Uuid;

use crate::db_pool;
use crate::models::Person;

use super::download_file;

pub async fn delete_person(person: &Person<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "DELETE FROM persons WHERE id = $1",
        person.id // $1
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_or_insert_person<'a>(
    tmdb_id: i32,
    tmdb_profile_path: Option<&str>,
    tmdb_profile_url: Option<Url>,
    imdb_id: Option<&str>,
    name: &str,
) -> sqlx::Result<Person<'a>> {
    if let Ok(person) = get_person_by_tmdb_id(tmdb_id).await {
        return Ok(person);
    }

    let db_pool = db_pool().await;

    let person = sqlx::query_as!(
        Person,
        "INSERT INTO persons (tmdb_id, tmdb_profile_path, imdb_id, name) VALUES ($1, $2, $3, $4) RETURNING *",
        tmdb_id,           // $1
        tmdb_profile_path, // $2
        imdb_id,           // $3
        name,              // $4
    )
    .fetch_one(db_pool)
    .await?;

    if let Some(source_url) = tmdb_profile_url
        && let Some(dest_url) = person.profile_image_path()
    {
        let _ = download_file(source_url, dest_url).await;
    }

    Ok(person)
}

pub async fn get_person_by_id<'a>(id: Uuid) -> sqlx::Result<Person<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Person,
        "SELECT * FROM persons WHERE id = $1 LIMIT 1",
        id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_person_by_tmdb_id<'a>(tmdb_id: i32) -> sqlx::Result<Person<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Person,
        "SELECT * FROM persons WHERE tmdb_id = $1 LIMIT 1",
        tmdb_id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_or_update_person(
    tmdb_id: i32,
    tmdb_profile_path: Option<&str>,
    tmdb_profile_url: Option<Url>,
    imdb_id: Option<&str>,
    name: &str,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    let person = sqlx::query_as!(
        Person,
        "INSERT INTO persons (tmdb_id, tmdb_profile_path, imdb_id, name) VALUES ($1, $2, $3, $4)
        ON CONFLICT (tmdb_id) DO UPDATE SET tmdb_profile_path = $2, imdb_id = $3, name = $4 RETURNING *",
        tmdb_id,           // $1
        tmdb_profile_path, // $2
        imdb_id,           // $3
        name,              // $4
    )
    .fetch_one(db_pool)
    .await?;

    if let Some(source_url) = tmdb_profile_url
        && let Some(dest_url) = person.profile_image_path()
    {
        let _ = download_file(source_url, dest_url).await;
    }

    Ok(())
}
