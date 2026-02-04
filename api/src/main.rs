use std::net::SocketAddr;

use async_graphql::extensions::apollo_persisted_queries::{ApolloPersistedQueries, LruCacheStorage};
use async_graphql::extensions::{ApolloTracing, Logger};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::{HeaderMap, Method};
use axum::response::{IntoResponse, Result};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_client_ip::ClientIp;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use tokio::net::TcpListener;
use tokio::try_join;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::Level;

use trailers_core::config::STORAGE_CONFIG;
use trailers_core::graphql::{GraphqlSchema, GraphqlSchemaExt};
use trailers_core::{Info, commands};

use crate::config::API_CONFIG;
use crate::constants::{ERROR_FORBIDDEN, HEADER_X_API_TOKEN};

mod config;
mod constants;

trait HttpError<T> {
    #[allow(clippy::result_large_err)]
    fn or_forbidden(self) -> Result<T>;
}

impl<T> HttpError<T> for Option<T> {
    fn or_forbidden(self) -> Result<T> {
        match self {
            Some(value) => Ok(value),
            None => Err(ERROR_FORBIDDEN.into()),
        }
    }
}

impl<T, E> HttpError<T> for Result<T, E> {
    fn or_forbidden(self) -> Result<T> {
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
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    ClientIp(client_ip): ClientIp,
    batch_request: GraphQLBatchRequest,
) -> Result<GraphQLResponse> {
    let api_token = headers
        .get(HEADER_X_API_TOKEN)
        .or_forbidden()?
        .to_str()
        .or_forbidden()?;

    if !API_CONFIG.tokens().contains(&api_token) {
        return Err(ERROR_FORBIDDEN.into());
    }

    let mut batch_request = batch_request.into_inner();

    batch_request = batch_request.data(client_ip);

    if let Some(TypedHeader(Authorization(bearer))) = authorization {
        let token = bearer.token().to_owned();

        if let Ok((session, user)) = try_join!(
            commands::get_session_by_token(token.clone()),
            commands::get_user_by_session_token(token)
        ) {
            batch_request = batch_request.data(session).data(user);
        }
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

    tracing_subscriber::fmt().with_max_level(tracing_level).init();

    let mut graphql_schema_builder = GraphqlSchema::builder()
        .extension(ApolloPersistedQueries::new(LruCacheStorage::new(1024)))
        .extension(Logger);

    graphql_schema_builder = if !cfg!(debug_assertions) {
        graphql_schema_builder.disable_introspection()
    } else {
        graphql_schema_builder.extension(ApolloTracing)
    };

    let graphql_schema = graphql_schema_builder.finish();

    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::POST])
        .allow_headers(Any);

    let mut router = Router::new()
        .route("/", get(get_index))
        .route("/graphql", post(post_graphql))
        .with_state(graphql_schema)
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http())
        .layer(API_CONFIG.client_ip_source.clone().into_extension());

    if API_CONFIG.serve_storage {
        router = router.nest_service("/storage", ServeDir::new(&STORAGE_CONFIG.path));
    }

    let api_address = &API_CONFIG.address;

    let listener = TcpListener::bind(&api_address).await.unwrap();

    tracing::info!("Listening on {api_address}");

    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
