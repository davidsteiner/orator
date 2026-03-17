mod api;

use api::generated::*;
use serde_json::json;
use std::sync::Arc;
use tennis_club_core::TennisClub;

#[tokio::main]
async fn main() {
    let api = Arc::new(TennisClub::new());

    let scalar_config = json!({ "url": "/openapi.yaml", "theme": "kepler" });
    let app = ApiBuilder::new()
        .members(MembersRouter::new(api.clone()))
        .courts(CourtsRouter::new(api.clone()))
        .bookings(BookingsRouter::new(api))
        .build()
        .route(
            "/openapi.yaml",
            axum::routing::get(|| async {
                (
                    [("content-type", "application/yaml")],
                    include_str!("../../tennis-club-core/tennis-club.yaml"),
                )
            }),
        )
        .route(
            "/docs",
            axum::routing::get(move || async move {
                axum::response::Html(scalar_api_reference::scalar_html_default(&scalar_config))
            }),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    println!("API docs at http://localhost:3000/docs");
    axum::serve(listener, app).await.unwrap();
}
