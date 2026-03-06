mod api;

use api::*;
use serde_json::json;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

struct TennisClub {
    members: Mutex<Vec<Member>>,
    courts: Mutex<Vec<Court>>,
    next_id: Mutex<i64>,
}

impl TennisClub {
    fn new() -> Self {
        let courts = vec![
            Court {
                id: 1,
                name: "Centre Court".to_string(),
                surface: Surface::Grass,
                indoor: Some(false),
            },
            Court {
                id: 2,
                name: "Court Philippe-Chatrier".to_string(),
                surface: Surface::Clay,
                indoor: Some(false),
            },
            Court {
                id: 3,
                name: "Indoor Hard Court".to_string(),
                surface: Surface::Hard,
                indoor: Some(true),
            },
        ];

        Self {
            members: Mutex::new(Vec::new()),
            courts: Mutex::new(courts),
            next_id: Mutex::new(1),
        }
    }
}

impl MembersApi for TennisClub {
    type Error = Infallible;

    async fn list_members(
        &self,
        _ctx: (),
        _params: ListMembersParams,
    ) -> Result<ListMembersResponse, Self::Error> {
        let members = self.members.lock().unwrap();
        Ok(ListMembersResponse::Ok(members.clone()))
    }

    async fn create_member(
        &self,
        _ctx: (),
        params: CreateMemberParams,
    ) -> Result<CreateMemberResponse, Self::Error> {
        let mut members = self.members.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();

        let member = Member {
            id: *next_id,
            first_name: params.body.first_name,
            last_name: params.body.last_name,
        };

        *next_id += 1;
        members.push(member.clone());

        Ok(CreateMemberResponse::Created(member))
    }

    async fn get_member(
        &self,
        _ctx: (),
        params: GetMemberParams,
    ) -> Result<GetMemberResponse, Self::Error> {
        let members = self.members.lock().unwrap();

        match members.iter().find(|m| m.id == params.member_id) {
            Some(member) => Ok(GetMemberResponse::Ok(member.clone())),
            None => Ok(GetMemberResponse::NotFound(Error {
                code: 404,
                message: format!("Member {} not found", params.member_id),
            })),
        }
    }

    async fn update_member(
        &self,
        _ctx: (),
        params: UpdateMemberParams,
    ) -> Result<UpdateMemberResponse, Self::Error> {
        let mut members = self.members.lock().unwrap();

        match members.iter_mut().find(|m| m.id == params.member_id) {
            Some(member) => {
                if let Some(first_name) = params.body.first_name {
                    member.first_name = first_name;
                }
                if let Some(last_name) = params.body.last_name {
                    member.last_name = last_name;
                }
                Ok(UpdateMemberResponse::Ok(member.clone()))
            }
            None => Ok(UpdateMemberResponse::NotFound(Error {
                code: 404,
                message: format!("Member {} not found", params.member_id),
            })),
        }
    }

    async fn delete_member(
        &self,
        _ctx: (),
        params: DeleteMemberParams,
    ) -> Result<DeleteMemberResponse, Self::Error> {
        let mut members = self.members.lock().unwrap();
        let len_before = members.len();
        members.retain(|m| m.id != params.member_id);

        if members.len() < len_before {
            Ok(DeleteMemberResponse::NoContent)
        } else {
            Ok(DeleteMemberResponse::NotFound(Error {
                code: 404,
                message: format!("Member {} not found", params.member_id),
            }))
        }
    }
}

impl CourtsApi for TennisClub {
    type Error = Infallible;

    async fn list_courts(
        &self,
        _ctx: (),
        _params: ListCourtsParams,
    ) -> Result<ListCourtsResponse, Self::Error> {
        let courts = self.courts.lock().unwrap();
        Ok(ListCourtsResponse::Ok(courts.clone()))
    }
}

#[tokio::main]
async fn main() {
    let api = Arc::new(TennisClub::new());

    let scalar_config = json!({ "url": "/openapi.yaml", "theme": "kepler" });
    let app = members_router(api.clone())
        .merge(courts_router(api))
        .route(
            "/openapi.yaml",
            axum::routing::get(|| async {
                (
                    [("content-type", "application/yaml")],
                    include_str!("../tennis-club.yaml"),
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
