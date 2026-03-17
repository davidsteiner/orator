use crate::api::generated::{
    CreateMemberResponse, DeleteMemberPath, DeleteMemberResponse, GetMemberPath,
    GetMemberPhotoPath, GetMemberPhotoResponse, GetMemberResponse, ListMembersCookie,
    ListMembersHeader, ListMembersQuery, ListMembersResponse, MembersApi, UpdateMemberPath,
    UpdateMemberResponse, UploadMemberPhotoPath, UploadMemberPhotoResponse,
};
use std::convert::Infallible;
use tennis_club_core::{TennisClub, domain};

impl MembersApi for TennisClub {
    type Error = Infallible;

    async fn list_members(
        &self,
        _ctx: (),
        query: ListMembersQuery,
        _header: ListMembersHeader,
        _cookie: ListMembersCookie,
    ) -> Result<ListMembersResponse, Self::Error> {
        let members = self.list_members(query.limit, query.offset);
        let members = members.into_iter().map(Into::into).collect();
        Ok(ListMembersResponse::Ok(members))
    }

    async fn create_member(
        &self,
        _ctx: (),
        body: crate::api::generated::NewMember,
    ) -> Result<CreateMemberResponse, Self::Error> {
        let new = domain::NewMember {
            first_name: body.first_name,
            last_name: body.last_name,
        };
        let member = self.create_member(new);
        Ok(CreateMemberResponse::Created(member.into()))
    }

    async fn get_member(
        &self,
        _ctx: (),
        path: GetMemberPath,
    ) -> Result<GetMemberResponse, Self::Error> {
        match self.get_member(path.member_id) {
            Some(member) => Ok(GetMemberResponse::Ok(member.into())),
            None => Ok(GetMemberResponse::NotFound(crate::api::generated::Error {
                code: 404,
                message: format!("Member {} not found", path.member_id),
            })),
        }
    }

    async fn update_member(
        &self,
        _ctx: (),
        path: UpdateMemberPath,
        body: crate::api::generated::UpdateMember,
    ) -> Result<UpdateMemberResponse, Self::Error> {
        let update = domain::UpdateMember {
            first_name: body.first_name,
            last_name: body.last_name,
        };
        match self.update_member(path.member_id, update) {
            Some(member) => Ok(UpdateMemberResponse::Ok(member.into())),
            None => Ok(UpdateMemberResponse::NotFound(
                crate::api::generated::Error {
                    code: 404,
                    message: format!("Member {} not found", path.member_id),
                },
            )),
        }
    }

    async fn delete_member(
        &self,
        _ctx: (),
        path: DeleteMemberPath,
    ) -> Result<DeleteMemberResponse, Self::Error> {
        if self.delete_member(path.member_id) {
            Ok(DeleteMemberResponse::NoContent)
        } else {
            Ok(DeleteMemberResponse::NotFound(
                crate::api::generated::Error {
                    code: 404,
                    message: format!("Member {} not found", path.member_id),
                },
            ))
        }
    }

    async fn get_member_photo(
        &self,
        _ctx: (),
        path: GetMemberPhotoPath,
    ) -> Result<GetMemberPhotoResponse, Self::Error> {
        Ok(GetMemberPhotoResponse::NotFound(
            crate::api::generated::Error {
                code: 404,
                message: format!("No photo for member {}", path.member_id),
            },
        ))
    }

    async fn upload_member_photo(
        &self,
        _ctx: (),
        _path: UploadMemberPhotoPath,
        _body: orator_axum::bytes::Bytes,
    ) -> Result<UploadMemberPhotoResponse, Self::Error> {
        Ok(UploadMemberPhotoResponse::NoContent)
    }
}

impl From<domain::Member> for crate::api::generated::Member {
    fn from(m: domain::Member) -> Self {
        Self {
            id: m.id,
            first_name: m.first_name,
            last_name: m.last_name,
        }
    }
}
