use crate::TennisClub;
use crate::api::generated::{
    CreateMemberResponse, DeleteMemberPath, DeleteMemberResponse, Error, GetMemberPath,
    GetMemberResponse, ListMembersQuery, ListMembersResponse, Member, MembersApi, NewMember,
    UpdateMember, UpdateMemberPath, UpdateMemberResponse,
};
use std::convert::Infallible;

impl MembersApi for TennisClub {
    type Error = Infallible;

    async fn list_members(
        &self,
        _ctx: (),
        query: ListMembersQuery,
    ) -> Result<ListMembersResponse, Self::Error> {
        let members = self.members.lock().unwrap();
        let offset = query.offset.unwrap_or(0) as usize;
        let result: Vec<_> = members
            .iter()
            .skip(offset)
            .take(query.limit.unwrap_or(i32::MAX) as usize)
            .cloned()
            .collect();
        Ok(ListMembersResponse::Ok(result))
    }

    async fn create_member(
        &self,
        _ctx: (),
        body: NewMember,
    ) -> Result<CreateMemberResponse, Self::Error> {
        let mut members = self.members.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();

        let member = Member {
            id: *next_id,
            first_name: body.first_name,
            last_name: body.last_name,
        };

        *next_id += 1;
        members.push(member.clone());

        Ok(CreateMemberResponse::Created(member))
    }

    async fn get_member(
        &self,
        _ctx: (),
        path: GetMemberPath,
    ) -> Result<GetMemberResponse, Self::Error> {
        let members = self.members.lock().unwrap();

        match members.iter().find(|m| m.id == path.member_id) {
            Some(member) => Ok(GetMemberResponse::Ok(member.clone())),
            None => Ok(GetMemberResponse::NotFound(Error {
                code: 404,
                message: format!("Member {} not found", path.member_id),
            })),
        }
    }

    async fn update_member(
        &self,
        _ctx: (),
        path: UpdateMemberPath,
        body: UpdateMember,
    ) -> Result<UpdateMemberResponse, Self::Error> {
        let mut members = self.members.lock().unwrap();

        match members.iter_mut().find(|m| m.id == path.member_id) {
            Some(member) => {
                if let Some(first_name) = body.first_name {
                    member.first_name = first_name;
                }
                if let Some(last_name) = body.last_name {
                    member.last_name = last_name;
                }
                Ok(UpdateMemberResponse::Ok(member.clone()))
            }
            None => Ok(UpdateMemberResponse::NotFound(Error {
                code: 404,
                message: format!("Member {} not found", path.member_id),
            })),
        }
    }

    async fn delete_member(
        &self,
        _ctx: (),
        path: DeleteMemberPath,
    ) -> Result<DeleteMemberResponse, Self::Error> {
        let mut members = self.members.lock().unwrap();
        let len_before = members.len();
        members.retain(|m| m.id != path.member_id);

        if members.len() < len_before {
            Ok(DeleteMemberResponse::NoContent)
        } else {
            Ok(DeleteMemberResponse::NotFound(Error {
                code: 404,
                message: format!("Member {} not found", path.member_id),
            }))
        }
    }
}
