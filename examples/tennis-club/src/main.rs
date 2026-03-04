include!(concat!(env!("OUT_DIR"), "/types.rs"));

use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use http::StatusCode;

struct TennisClub {
    members: Mutex<Vec<Member>>,
    next_id: Mutex<i64>,
}

impl TennisClub {
    fn new() -> Self {
        Self {
            members: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

async fn list_members(State(state): State<Arc<TennisClub>>) -> impl IntoResponse {
    let members = state.members.lock().unwrap();
    Json(members.clone())
}

async fn create_member(
    State(state): State<Arc<TennisClub>>,
    Json(body): Json<NewMember>,
) -> impl IntoResponse {
    let mut members = state.members.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();

    let member = Member {
        id: *next_id,
        first_name: body.first_name,
        last_name: body.last_name,
    };

    *next_id += 1;
    members.push(member.clone());

    (StatusCode::CREATED, Json(member))
}

async fn get_member(
    State(state): State<Arc<TennisClub>>,
    Path(member_id): Path<i64>,
) -> Result<Json<Member>, (StatusCode, Json<Error>)> {
    let members = state.members.lock().unwrap();

    match members.iter().find(|m| m.id == member_id) {
        Some(member) => Ok(Json(member.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(Error {
                code: 404,
                message: format!("Member {member_id} not found"),
            }),
        )),
    }
}

async fn update_member(
    State(state): State<Arc<TennisClub>>,
    Path(member_id): Path<i64>,
    Json(body): Json<UpdateMember>,
) -> Result<Json<Member>, (StatusCode, Json<Error>)> {
    let mut members = state.members.lock().unwrap();

    match members.iter_mut().find(|m| m.id == member_id) {
        Some(member) => {
            if let Some(first_name) = body.first_name {
                member.first_name = first_name;
            }
            if let Some(last_name) = body.last_name {
                member.last_name = last_name;
            }
            Ok(Json(member.clone()))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(Error {
                code: 404,
                message: format!("Member {member_id} not found"),
            }),
        )),
    }
}

async fn delete_member(
    State(state): State<Arc<TennisClub>>,
    Path(member_id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<Error>)> {
    let mut members = state.members.lock().unwrap();
    let len_before = members.len();
    members.retain(|m| m.id != member_id);

    if members.len() < len_before {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(Error {
                code: 404,
                message: format!("Member {member_id} not found"),
            }),
        ))
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(TennisClub::new());

    let app = Router::new()
        .route("/members", get(list_members).post(create_member))
        .route(
            "/members/{member_id}",
            get(get_member).patch(update_member).delete(delete_member),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
