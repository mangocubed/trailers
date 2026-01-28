use async_graphql::extensions::apollo_persisted_queries::{
    ApolloPersistedQueries, LruCacheStorage,
};
use async_graphql::extensions::{ApolloTracing, Tracing};
use async_graphql_axum::GraphQL;
use axum::response::IntoResponse;
use axum::routing::{get, post_service};
use axum::{Json, Router};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::Level;

use trailers_core::Info;
use trailers_core::config::API_CONFIG;
use trailers_core::graphql::{GraphqlSchema, GraphqlSchemaExt};

async fn get_index() -> impl IntoResponse {
    Json(Info::default())
}

#[tokio::main]
async fn main() {
    let tracing_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(tracing_level)
        .init();

    let mut graphql_schema_builder = GraphqlSchema::builder()
        .extension(ApolloPersistedQueries::new(LruCacheStorage::new(1024)))
        .extension(Tracing);

    graphql_schema_builder = if !cfg!(debug_assertions) {
        graphql_schema_builder.disable_introspection()
    } else {
        graphql_schema_builder.extension(ApolloTracing)
    };

    let graphql_schema = graphql_schema_builder.finish();

    let router = Router::new()
        .route("/", get(get_index))
        .route("/graphql", post_service(GraphQL::new(graphql_schema)))
        .layer(TraceLayer::new_for_http());

    let api_address = &API_CONFIG.address;

    let listener = TcpListener::bind(&api_address).await.unwrap();

    tracing::info!("Listening on {api_address}");

    axum::serve(listener, router).await.unwrap();
}
