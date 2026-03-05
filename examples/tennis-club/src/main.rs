include!(concat!(env!("OUT_DIR"), "/types.rs"));

mod api;

use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use api::*;

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

impl TennisClubApi for TennisClub {
    type Error = Infallible;

    async fn list_members(&self, _ctx: (), _params: ListMembersParams) -> Result<ListMembersResponse, Self::Error> {
        let members = self.members.lock().unwrap();
        Ok(ListMembersResponse::Ok(members.clone()))
    }

    async fn create_member(&self, _ctx: (), params: CreateMemberParams) -> Result<CreateMemberResponse, Self::Error> {
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

    async fn get_member(&self, _ctx: (), params: GetMemberParams) -> Result<GetMemberResponse, Self::Error> {
        let members = self.members.lock().unwrap();

        match members.iter().find(|m| m.id == params.member_id) {
            Some(member) => Ok(GetMemberResponse::Ok(member.clone())),
            None => Ok(GetMemberResponse::NotFound(Error {
                code: 404,
                message: format!("Member {} not found", params.member_id),
            })),
        }
    }

    async fn update_member(&self, _ctx: (), params: UpdateMemberParams) -> Result<UpdateMemberResponse, Self::Error> {
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

    async fn delete_member(&self, _ctx: (), params: DeleteMemberParams) -> Result<DeleteMemberResponse, Self::Error> {
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

#[tokio::main]
async fn main() {
    let api = Arc::new(TennisClub::new());
    let app = tennis_club_router(api);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
