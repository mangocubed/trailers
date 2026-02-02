use std::str::FromStr;

use chrono::{NaiveDate, TimeDelta};
use reqwest::StatusCode;

use trailers_core::enums::TitleMediaType;
use trailers_core::models::Title;
use trailers_core::{commands, enums::TitleCrewJob};

use crate::tmdb::{Tmdb, TmdbGenre};

pub async fn populate_movies(end_date: Option<NaiveDate>, start_date: Option<NaiveDate>) -> anyhow::Result<()> {
    let mut page = 1;
    let mut total_pages = 1;
    let tmdb = Tmdb::new();

    while page <= total_pages {
        let tmdb_changes = tmdb.movie_changes(page, end_date, start_date).await?;

        for tmdb_changes_item in tmdb_changes.results {
            if tmdb_changes_item.adult.is_none() {
                continue;
            }

            let Some(tmdb_movie_id) = tmdb_changes_item.id else {
                continue;
            };

            let tmdb_movie_result = tmdb.movie(tmdb_movie_id).await;

            match tmdb_movie_result {
                Ok(tmdb_movie) => {
                    let media_type = if tmdb_movie.runtime == 0 || tmdb_movie.runtime > 40 {
                        TitleMediaType::Movie
                    } else {
                        TitleMediaType::Short
                    };

                    let runtime = if tmdb_movie.runtime > 0 {
                        Some(TimeDelta::minutes(tmdb_movie.runtime))
                    } else {
                        None
                    };

                    let release_date = tmdb_movie
                        .release_date
                        .as_ref()
                        .and_then(|value| NaiveDate::from_str(&value).ok());

                    let result = commands::insert_or_update_title(
                        media_type,
                        tmdb_movie.id,
                        tmdb_movie.backdrop_path.as_deref(),
                        tmdb_movie.backdrop_url(),
                        tmdb_movie.poster_path.as_deref(),
                        tmdb_movie.poster_url(),
                        tmdb_movie.imdb_id.as_deref(),
                        &tmdb_movie.title,
                        &tmdb_movie.overview,
                        &tmdb_movie.original_language,
                        runtime,
                        tmdb_movie.adult.unwrap_or(false),
                        release_date,
                    )
                    .await;

                    if let Ok(title) = result {
                        let _ = populate_title_extras(&title, &tmdb_movie.genres).await;
                    }
                }

                Err(error) => {
                    if error.status() == Some(StatusCode::NOT_FOUND)
                        && let Ok(title) = commands::get_title_by_tmdb_id(TitleMediaType::Movie, tmdb_movie_id)
                            .await
                            .or(commands::get_title_by_tmdb_id(TitleMediaType::Short, tmdb_movie_id).await)
                    {
                        let _ = commands::delete_title(&title).await;
                    }
                }
            }
        }

        total_pages = if tmdb_changes.total_pages <= 500 {
            tmdb_changes.total_pages
        } else {
            500
        };
        page += 1;
    }

    Ok(())
}

pub async fn populate_persons(end_date: Option<NaiveDate>, start_date: Option<NaiveDate>) -> anyhow::Result<()> {
    let mut page = 1;
    let mut total_pages = 1;
    let tmdb = Tmdb::new();

    while page <= total_pages {
        let tmdb_changes = tmdb.person_changes(page, end_date, start_date).await?;

        for tmdb_changes_item in tmdb_changes.results {
            if tmdb_changes_item.adult.is_none() {
                continue;
            }

            let Some(tmdb_person_id) = tmdb_changes_item.id else {
                continue;
            };

            let tmdb_person_result = tmdb.person(tmdb_person_id).await;

            match tmdb_person_result {
                Ok(tmdb_person) => {
                    let _ = commands::insert_or_update_person(
                        tmdb_person.id,
                        tmdb_person.profile_path.as_deref(),
                        tmdb_person.profile_url(),
                        tmdb_person.imdb_id.as_deref(),
                        &tmdb_person.name,
                    )
                    .await;
                }
                Err(error) => {
                    if error.status() == Some(StatusCode::NOT_FOUND)
                        && let Ok(person) = commands::get_person_by_tmdb_id(tmdb_person_id).await
                    {
                        let _ = commands::delete_person(&person).await;
                    }
                }
            }
        }

        total_pages = if tmdb_changes.total_pages <= 500 {
            tmdb_changes.total_pages
        } else {
            500
        };
        page += 1;
    }

    Ok(())
}

pub async fn populate_series(end_date: Option<NaiveDate>, start_date: Option<NaiveDate>) -> anyhow::Result<()> {
    let mut page = 1;
    let mut total_pages = 1;
    let tmdb = Tmdb::new();

    while page <= total_pages {
        let tmdb_changes = tmdb.tv_changes(page, end_date, start_date).await?;

        for tmdb_changes_item in tmdb_changes.results {
            if tmdb_changes_item.adult.is_none() {
                continue;
            }

            let Some(tmdb_tv_id) = tmdb_changes_item.id else {
                continue;
            };

            let tmdb_tv_result = tmdb.tv(tmdb_tv_id).await;

            match tmdb_tv_result {
                Ok(tmdb_tv) => {
                    let first_air_date = tmdb_tv
                        .first_air_date
                        .as_ref()
                        .and_then(|value| NaiveDate::from_str(&value).ok());

                    let result = commands::insert_or_update_title(
                        TitleMediaType::Series,
                        tmdb_tv.id,
                        tmdb_tv.backdrop_path.as_deref(),
                        tmdb_tv.backdrop_url(),
                        tmdb_tv.poster_path.as_deref(),
                        tmdb_tv.poster_url(),
                        tmdb_tv.imdb_id.as_deref(),
                        &tmdb_tv.name,
                        &tmdb_tv.overview,
                        &tmdb_tv.original_language,
                        None,
                        tmdb_tv.adult.unwrap_or(false),
                        first_air_date,
                    )
                    .await;

                    if let Ok(title) = result {
                        let _ = populate_title_extras(&title, &tmdb_tv.genres).await;
                    }
                }

                Err(error) => {
                    if error.status() == Some(StatusCode::NOT_FOUND)
                        && let Ok(title) = commands::get_title_by_tmdb_id(TitleMediaType::Series, tmdb_tv_id).await
                    {
                        let _ = commands::delete_title(&title).await;
                    }
                }
            }
        }

        total_pages = if tmdb_changes.total_pages <= 500 {
            tmdb_changes.total_pages
        } else {
            500
        };
        page += 1;
    }

    Ok(())
}

async fn populate_title_cast_and_crew(title: &Title<'_>) -> anyhow::Result<()> {
    let tmdb = Tmdb::new();

    let tmdb_credits = match title.media_type {
        TitleMediaType::Series => tmdb.tv_credits(title.tmdb_id).await?,
        _ => tmdb.movie_credits(title.tmdb_id).await?,
    };

    for tmdb_cast in tmdb_credits.cast {
        let Ok(person) =
            commands::get_or_insert_person(tmdb_cast.id, tmdb_cast.profile_path.as_deref(), None, &tmdb_cast.name)
                .await
        else {
            continue;
        };

        let _ = commands::insert_title_cast(title, &person, &tmdb_cast.credit_id, &tmdb_cast.character).await;
    }

    for tmdb_crew in tmdb_credits.crew {
        if tmdb_crew.job != "Director" {
            continue;
        }

        let Ok(person) =
            commands::get_or_insert_person(tmdb_crew.id, tmdb_crew.profile_path.as_deref(), None, &tmdb_crew.name)
                .await
        else {
            continue;
        };

        let _ = commands::insert_title_crew(title, &person, &tmdb_crew.credit_id, TitleCrewJob::Director).await;
    }

    Ok(())
}

async fn populate_title_extras(title: &Title<'_>, tmdb_genres: &[TmdbGenre<'_>]) -> anyhow::Result<()> {
    let _ = populate_title_cast_and_crew(title).await;
    let _ = populate_title_genres(title, tmdb_genres).await;
    let _ = populate_title_keywords(title).await;

    Ok(())
}

async fn populate_title_genres(title: &Title<'_>, tmdb_genres: &[TmdbGenre<'_>]) -> anyhow::Result<()> {
    for tmdb_genre in tmdb_genres {
        let Ok(genre) = commands::insert_genre(tmdb_genre.id, &tmdb_genre.name).await else {
            continue;
        };

        let _ = commands::insert_title_genre(title, &genre).await;
    }

    Ok(())
}

async fn populate_title_keywords(title: &Title<'_>) -> anyhow::Result<()> {
    let tmdb = Tmdb::new();

    let tmdb_keywords = match title.media_type {
        TitleMediaType::Series => tmdb.tv_keywords(title.tmdb_id).await?,
        _ => tmdb.movie_keywords(title.tmdb_id).await?,
    };

    for tmdb_keyword in tmdb_keywords.keywords() {
        let Ok(keyword) = commands::insert_keyword(tmdb_keyword.id, &tmdb_keyword.name).await else {
            continue;
        };

        let _ = commands::insert_title_keyword(title, &keyword).await;
    }

    Ok(())
}
