use async_graphql::extensions::apollo_persisted_queries::{
    ApolloPersistedQueries, LruCacheStorage,
};
use async_graphql::extensions::{ApolloTracing, Logger};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, StatusCode};
use axum::response::{ErrorResponse, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::Level;

use trailers_core::Info;
use trailers_core::config::API_CONFIG;
use trailers_core::graphql::{GraphqlSchema, GraphqlSchemaExt};

const ERROR_FORBIDDEN: (StatusCode, &str) = (StatusCode::FORBIDDEN, "Forbidden");
const HEADER_X_TOKEN: HeaderName = HeaderName::from_static("x-token");

trait HttpError<T> {
    fn or_forbidden(self) -> Result<T, ErrorResponse>;
}

impl<T> HttpError<T> for Option<T> {
    fn or_forbidden(self) -> Result<T, ErrorResponse> {
        match self {
            Some(value) => Ok(value),
            None => Err(ERROR_FORBIDDEN.into()),
        }
    }
}

impl<T, E> HttpError<T> for Result<T, E> {
    fn or_forbidden(self) -> Result<T, ErrorResponse> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(ERROR_FORBIDDEN.into()),
        }
    }
}

async fn get_index() -> impl IntoResponse {
    Json(Info::default())
}

async fn post_graphql(
    State(schema): State<GraphqlSchema>,
    headers: HeaderMap,
    batch_request: GraphQLBatchRequest,
) -> Result<GraphQLResponse, ErrorResponse> {
    let batch_request = batch_request.into_inner();

    let x_token = headers
        .get(HEADER_X_TOKEN)
        .or_forbidden()?
        .to_str()
        .or_forbidden()?;

    if API_CONFIG.tokens().contains(&x_token) && API_CONFIG.old_tokens().contains(&x_token) {
        return Err(ERROR_FORBIDDEN.into());
    }

    Ok(schema.execute_batch(batch_request).await.into())
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
        .extension(Logger);

    graphql_schema_builder = if !cfg!(debug_assertions) {
        graphql_schema_builder.disable_introspection()
    } else {
        graphql_schema_builder.extension(ApolloTracing)
    };

    let graphql_schema = graphql_schema_builder.finish();

    let router = Router::new()
        .route("/", get(get_index))
        .route("/graphql", post(post_graphql))
        .with_state(graphql_schema)
        .layer(TraceLayer::new_for_http());

    let api_address = &API_CONFIG.address;

    let listener = TcpListener::bind(&api_address).await.unwrap();

    tracing::info!("Listening on {api_address}");

    axum::serve(listener, router).await.unwrap();
}
