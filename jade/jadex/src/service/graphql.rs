//! GraphQL service for the Jadex runtime

use async_graphql::{http::GraphiQLSource, ObjectType, Schema, SubscriptionType};
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use std::{any::Any, net::SocketAddr};
use tokio::net::TcpListener;

/// Start the GraphQL service
pub async fn start<Query, Mutation, Subscription, Data>(
    query: Query,
    mutation: Mutation,
    subscription: Subscription,
    data: Data,
    address: SocketAddr,
) -> anyhow::Result<()>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    Data: Send + Sync + Any,
{
    let schema = Schema::build(query, mutation, subscription)
        .data(data)
        .disable_introspection()
        .finish();

    let app = Router::new()
        .route(
            "/",
            get(graphiql).post_service(GraphQL::new(schema.clone())),
        )
        .route_service("/ws", GraphQLSubscription::new(schema));

    axum::serve(TcpListener::bind(address).await?, app).await?;
    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}
