#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[derive(sqlx::Type, Clone, Copy, Eq, PartialEq)]
#[sqlx(type_name = "title_crew_job", rename_all = "snake_case")]
pub enum TitleCrewJob {
    Director,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[derive(sqlx::Type, Clone, Copy, Eq, PartialEq)]
#[sqlx(type_name = "title_media_type", rename_all = "snake_case")]
pub enum TitleMediaType {
    Movie,
    Series,
    Short,
}

#[derive(sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "video_orientation", rename_all = "snake_case")]
pub enum VideoOrientation {
    Landscape,
    Portrait,
}

impl VideoOrientation {
    pub fn from_aspect_ratio(value: f32) -> Self {
        if value > 1.0 {
            VideoOrientation::Landscape
        } else {
            VideoOrientation::Portrait
        }
    }
}

#[derive(sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "video_source", rename_all = "snake_case")]
pub enum VideoSource {
    Youtube,
}

#[derive(sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "video_type", rename_all = "snake_case")]
pub enum VideoType {
    Teaser,
    Trailer,
}
