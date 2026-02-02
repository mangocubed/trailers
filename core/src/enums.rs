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
