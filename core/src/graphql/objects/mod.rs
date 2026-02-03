use async_graphql::{ID, Object, Result};
use chrono::{DateTime, Utc};
use url::Url;

use crate::enums::TitleCrewJob;
use crate::models::{Genre, Keyword, Person, Session, TitleCast, TitleCrew, User, UserTitleTie, Video};
use crate::{Info, commands};

mod title_object;

pub use title_object::*;

pub struct InfoObject(pub Info);

#[Object]
impl InfoObject {
    async fn built_at(&self) -> DateTime<Utc> {
        self.0.built_at
    }

    async fn version(&self) -> &str {
        &self.0.version
    }
}

pub struct GenreObject<'a>(pub Genre<'a>);

#[Object]
impl GenreObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct KeywordObject<'a>(pub Keyword<'a>);

#[Object]
impl KeywordObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct PersonObject<'a>(pub Person<'a>);

#[Object]
impl PersonObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn profile_image_url(&self) -> Option<Url> {
        self.0.profile_image_url()
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct SessionObject<'a>(pub Session<'a>);

#[Object]
impl SessionObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn user(&self) -> Result<UserObject<'_>> {
        Ok(UserObject(self.0.user().await?))
    }

    async fn token(&self) -> &str {
        &self.0.token
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct TitleCastObject<'a>(pub TitleCast<'a>);

#[Object]
impl TitleCastObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn person(&self) -> Result<PersonObject<'_>> {
        Ok(commands::get_person_by_id(self.0.person_id).await.map(PersonObject)?)
    }

    async fn character_name(&self) -> &str {
        &self.0.character_name
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct TitleCrewObject<'a>(pub TitleCrew<'a>);

#[Object]
impl TitleCrewObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn person(&self) -> Result<PersonObject<'_>> {
        Ok(commands::get_person_by_id(self.0.person_id).await.map(PersonObject)?)
    }

    async fn job(&self) -> TitleCrewJob {
        self.0.job
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct UserObject<'a>(pub User<'a>);

#[Object]
impl UserObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn username(&self) -> &str {
        &self.0.username
    }

    async fn display_name(&self) -> &str {
        &self.0.display_name
    }

    async fn full_name(&self) -> &str {
        &self.0.full_name
    }

    async fn initials(&self) -> String {
        self.0.initials()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct UserTitleTieObject(pub UserTitleTie);

#[Object]
impl UserTitleTieObject {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn title(&self) -> Result<TitleObject<'_>> {
        Ok(self.0.title().await.map(TitleObject)?)
    }

    async fn is_bookmarked(&self) -> bool {
        self.0.bookmarked_at.is_some()
    }

    async fn is_liked(&self) -> bool {
        self.0.liked_at.is_some()
    }

    async fn is_watched(&self) -> bool {
        self.0.watched_at.is_some()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct VideoObject<'a>(pub Video<'a>);

#[Object]
impl VideoObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn title(&self) -> Result<TitleObject<'_>> {
        Ok(self.0.title().await.map(TitleObject)?)
    }

    async fn url(&self) -> Url {
        self.0.url()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
