use std::borrow::Cow;
use std::str::FromStr;

use chrono::{NaiveDate, TimeDelta, Utc};
use reqwest::StatusCode;

use trailers_core::commands;
use trailers_core::enums::{TitleCrewJob, TitleMediaType, VideoSource, VideoType};
use trailers_core::jobs::PopulateJob;
use trailers_core::models::Title;

use crate::tmdb::{Tmdb, TmdbGenre};

pub async fn populate_movies(job: &PopulateJob) -> anyhow::Result<()> {
    let mut page = 1;
    let mut total_pages = 1;
    let tmdb = Tmdb::new();

    while page <= total_pages {
        let tmdb_items = if let Some(query) = &job.query {
            tmdb.search_movie(page, query).await?
        } else {
            tmdb.movie_changes(page, job.end_date, job.start_date).await?
        };

        for tmdb_item in tmdb_items.results {
            if tmdb_item.adult.unwrap_or_default()
                || (job.query.is_some()
                    && commands::get_title_by_tmdb_id(TitleMediaType::Movie, tmdb_item.id)
                        .await
                        .or(commands::get_title_by_tmdb_id(TitleMediaType::Short, tmdb_item.id).await)
                        .is_ok())
            {
                continue;
            }

            let tmdb_movie_result = tmdb.movie(tmdb_item.id).await;

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
                        .and_then(|value| NaiveDate::from_str(value).ok());

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
                        && let Ok(title) = commands::get_title_by_tmdb_id(TitleMediaType::Movie, tmdb_item.id)
                            .await
                            .or(commands::get_title_by_tmdb_id(TitleMediaType::Short, tmdb_item.id).await)
                    {
                        let _ = commands::delete_title(&title).await;
                    }
                }
            }
        }

        total_pages = if tmdb_items.total_pages <= 500 {
            tmdb_items.total_pages
        } else {
            500
        };
        page += 1;
    }

    Ok(())
}

pub async fn populate_persons(job: &PopulateJob) -> anyhow::Result<()> {
    let mut page = 1;
    let mut total_pages = 1;
    let tmdb = Tmdb::new();

    while page <= total_pages {
        let tmdb_items = if let Some(query) = &job.query {
            tmdb.search_person(page, query).await?
        } else {
            tmdb.person_changes(page, job.end_date, job.start_date).await?
        };

        for tmdb_item in tmdb_items.results {
            if tmdb_item.adult.unwrap_or_default()
                || (job.query.is_some() && commands::get_person_by_tmdb_id(tmdb_item.id).await.is_ok())
            {
                continue;
            }

            let tmdb_person_result = tmdb.person(tmdb_item.id).await;

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
                        && let Ok(person) = commands::get_person_by_tmdb_id(tmdb_item.id).await
                    {
                        let _ = commands::delete_person(&person).await;
                    }
                }
            }
        }

        total_pages = if tmdb_items.total_pages <= 500 {
            tmdb_items.total_pages
        } else {
            500
        };
        page += 1;
    }

    Ok(())
}

pub async fn populate_series(job: &PopulateJob) -> anyhow::Result<()> {
    let mut page = 1;
    let mut total_pages = 1;
    let tmdb = Tmdb::new();

    while page <= total_pages {
        let tmdb_items = if let Some(query) = &job.query {
            tmdb.search_tv(page, query).await?
        } else {
            tmdb.tv_changes(page, job.end_date, job.start_date).await?
        };

        for tmdb_item in tmdb_items.results {
            if tmdb_item.adult.unwrap_or_default()
                || (job.query.is_some()
                    && commands::get_title_by_tmdb_id(TitleMediaType::Series, tmdb_item.id)
                        .await
                        .is_ok())
            {
                continue;
            }

            let tmdb_tv_result = tmdb.tv(tmdb_item.id).await;

            match tmdb_tv_result {
                Ok(tmdb_tv) => {
                    let first_air_date = tmdb_tv
                        .first_air_date
                        .as_ref()
                        .and_then(|value| NaiveDate::from_str(value).ok());

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
                        && let Ok(title) = commands::get_title_by_tmdb_id(TitleMediaType::Series, tmdb_item.id).await
                    {
                        let _ = commands::delete_title(&title).await;
                    }
                }
            }
        }

        total_pages = if tmdb_items.total_pages <= 500 {
            tmdb_items.total_pages
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
        let Ok(person) = commands::get_or_insert_person(
            tmdb_cast.id,
            tmdb_cast.profile_path.as_deref(),
            tmdb_cast.profile_url(),
            None,
            &tmdb_cast.name,
        )
        .await
        else {
            continue;
        };

        let _ = commands::insert_or_update_title_cast(
            title,
            &person,
            &tmdb_cast.credit_id,
            &tmdb_cast.character,
            tmdb_cast.order,
        )
        .await;
    }

    for tmdb_crew in tmdb_credits.crew {
        if tmdb_crew.job != "Director" {
            continue;
        }

        let Ok(person) = commands::get_or_insert_person(
            tmdb_crew.id,
            tmdb_crew.profile_path.as_deref(),
            tmdb_crew.profile_url(),
            None,
            &tmdb_crew.name,
        )
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
    let _ = populate_title_watch_providers(title).await;
    let _ = populate_videos(title).await;

    Ok(())
}

async fn populate_title_genres(title: &Title<'_>, tmdb_genres: &[TmdbGenre<'_>]) -> anyhow::Result<()> {
    for tmdb_genre in tmdb_genres {
        let Ok(genre) = commands::get_or_insert_genre(tmdb_genre.id, &tmdb_genre.name).await else {
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
        let Ok(keyword) = commands::get_or_insert_keyword(tmdb_keyword.id, &tmdb_keyword.name).await else {
            continue;
        };

        let _ = commands::insert_title_keyword(title, &keyword).await;
    }

    Ok(())
}

pub async fn populate_title_watch_providers(title: &Title<'_>) -> anyhow::Result<()> {
    let tmdb = Tmdb::new();

    let results = match title.media_type {
        TitleMediaType::Series => tmdb.tv_watch_providers(title.tmdb_id).await?.results,
        _ => tmdb.movie_watch_providers(title.tmdb_id).await?.results,
    };

    for result in &results {
        let mut tmdb_watch_providers = result.1.flatrate.clone().unwrap_or_default();

        tmdb_watch_providers.append(&mut result.1.ads.clone().unwrap_or_default());

        for tmdb_watch_provider in tmdb_watch_providers {
            let Ok(watch_provider) = commands::get_or_insert_watch_provider(
                tmdb_watch_provider.provider_id,
                &tmdb_watch_provider.provider_name,
                tmdb_watch_provider.logo_path.as_deref(),
                tmdb_watch_provider.logo_url(),
            )
            .await
            else {
                continue;
            };

            let mut country_codes = results
                .clone()
                .into_iter()
                .filter(|r| {
                    let mut tmdb_watch_providers = r.1.flatrate.clone().unwrap_or_default();

                    tmdb_watch_providers.append(&mut r.1.ads.clone().unwrap_or_default());

                    tmdb_watch_providers
                        .into_iter()
                        .any(|twp| twp.provider_id == tmdb_watch_provider.provider_id)
                })
                .map(|r| r.0.to_string())
                .collect::<Vec<_>>();

            country_codes.sort();

            let _ = commands::insert_or_update_title_watch_provider(title, &watch_provider, &country_codes).await;
        }
    }

    Ok(())
}

pub async fn populate_videos(title: &Title<'_>) -> anyhow::Result<()> {
    let tmdb = Tmdb::new();

    let mut tmdb_videos = match title.media_type {
        TitleMediaType::Series => tmdb.tv_videos(title.tmdb_id).await?,
        _ => tmdb.movie_videos(title.tmdb_id).await?,
    }
    .results;

    tmdb_videos.retain(|tmdb_video| {
        [Cow::Borrowed("Teaser"), Cow::Borrowed("Trailer")].contains(&tmdb_video.r#type)
            && tmdb_video.site == "YouTube"
            && tmdb_video.official
    });

    tmdb_videos.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    for tmdb_video in tmdb_videos {
        let video_type = if tmdb_video.r#type == "Teaser" {
            VideoType::Teaser
        } else {
            VideoType::Trailer
        };

        let result = commands::insert_video(
            title,
            &tmdb_video.id,
            VideoSource::Youtube,
            &tmdb_video.key,
            &tmdb_video.name,
            video_type,
            &tmdb_video.iso_639_1,
            tmdb_video.published_at.unwrap_or(Utc::now()),
        )
        .await;

        if result.is_ok() {
            break;
        }
    }

    Ok(())
}
