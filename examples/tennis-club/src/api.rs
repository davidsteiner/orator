#![allow(dead_code)]

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{FromRequestParts, Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use http::StatusCode;

use crate::{Error, Member, NewMember, UpdateMember};

pub enum ListMembersResponse {
    Ok(Vec<Member>),
    Unauthorized(Error),
}

pub enum CreateMemberResponse {
    Created(Member),
    UnprocessableEntity(Error),
}

pub enum GetMemberResponse {
    Ok(Member),
    NotFound(Error),
}

pub enum UpdateMemberResponse {
    Ok(Member),
    NotFound(Error),
}

pub enum DeleteMemberResponse {
    NoContent,
    NotFound(Error),
}

impl IntoResponse for ListMembersResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Ok(members) => (StatusCode::OK, Json(members)).into_response(),
            Self::Unauthorized(err) => (StatusCode::UNAUTHORIZED, Json(err)).into_response(),
        }
    }
}

impl IntoResponse for CreateMemberResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Created(member) => (StatusCode::CREATED, Json(member)).into_response(),
            Self::UnprocessableEntity(err) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(err)).into_response()
            }
        }
    }
}

impl IntoResponse for GetMemberResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Ok(member) => (StatusCode::OK, Json(member)).into_response(),
            Self::NotFound(err) => (StatusCode::NOT_FOUND, Json(err)).into_response(),
        }
    }
}

impl IntoResponse for UpdateMemberResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Ok(member) => (StatusCode::OK, Json(member)).into_response(),
            Self::NotFound(err) => (StatusCode::NOT_FOUND, Json(err)).into_response(),
        }
    }
}

impl IntoResponse for DeleteMemberResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(err) => (StatusCode::NOT_FOUND, Json(err)).into_response(),
        }
    }
}

pub struct ListMembersParams;

pub struct CreateMemberParams {
    pub body: NewMember,
}

pub struct GetMemberParams {
    pub member_id: i64,
}

pub struct UpdateMemberParams {
    pub member_id: i64,
    pub body: UpdateMember,
}

pub struct DeleteMemberParams {
    pub member_id: i64,
}

pub trait MembersApi<Ctx = ()>: Send + Sync + 'static {
    type Error: Send;

    fn list_members(
        &self,
        ctx: Ctx,
        params: ListMembersParams,
    ) -> impl Future<Output = Result<ListMembersResponse, Self::Error>> + Send;

    fn create_member(
        &self,
        ctx: Ctx,
        params: CreateMemberParams,
    ) -> impl Future<Output = Result<CreateMemberResponse, Self::Error>> + Send;

    fn get_member(
        &self,
        ctx: Ctx,
        params: GetMemberParams,
    ) -> impl Future<Output = Result<GetMemberResponse, Self::Error>> + Send;

    fn update_member(
        &self,
        ctx: Ctx,
        params: UpdateMemberParams,
    ) -> impl Future<Output = Result<UpdateMemberResponse, Self::Error>> + Send;

    fn delete_member(
        &self,
        ctx: Ctx,
        params: DeleteMemberParams,
    ) -> impl Future<Output = Result<DeleteMemberResponse, Self::Error>> + Send;
}

// axum glue for tag: "members"

async fn handle_list_members<T, Ctx>(
    State(api): State<Arc<T>>,
    ctx: Ctx,
) -> Result<ListMembersResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.list_members(ctx, ListMembersParams).await
}

async fn handle_create_member<T, Ctx>(
    State(api): State<Arc<T>>,
    ctx: Ctx,
    Json(body): Json<NewMember>,
) -> Result<CreateMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.create_member(ctx, CreateMemberParams { body }).await
}

async fn handle_get_member<T, Ctx>(
    State(api): State<Arc<T>>,
    ctx: Ctx,
    Path(member_id): Path<i64>,
) -> Result<GetMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.get_member(ctx, GetMemberParams { member_id }).await
}

async fn handle_update_member<T, Ctx>(
    State(api): State<Arc<T>>,
    ctx: Ctx,
    Path(member_id): Path<i64>,
    Json(body): Json<UpdateMember>,
) -> Result<UpdateMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.update_member(ctx, UpdateMemberParams { member_id, body })
        .await
}

async fn handle_delete_member<T, Ctx>(
    State(api): State<Arc<T>>,
    ctx: Ctx,
    Path(member_id): Path<i64>,
) -> Result<DeleteMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.delete_member(ctx, DeleteMemberParams { member_id })
        .await
}

pub fn members_router<T, Ctx>(api: Arc<T>) -> Router
where
    T: MembersApi<Ctx>,
    T::Error: IntoResponse,
    Ctx: FromRequestParts<Arc<T>> + Send + 'static,
{
    Router::new()
        .route(
            "/members",
            get(handle_list_members::<T, Ctx>).post(handle_create_member::<T, Ctx>),
        )
        .route(
            "/members/{member_id}",
            get(handle_get_member::<T, Ctx>)
                .patch(handle_update_member::<T, Ctx>)
                .delete(handle_delete_member::<T, Ctx>),
        )
        .with_state(api)
}
