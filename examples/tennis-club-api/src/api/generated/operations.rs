#![allow(dead_code, unused_imports, clippy::redundant_field_names)]
use orator_axum::serde;
/// List all bookings
#[derive(Debug)]
pub enum ListBookingsResponse {
    /// A list of bookings
    Ok(Vec<super::types::Booking>),
    /// Unauthorized
    Unauthorized(super::types::Error),
}
pub trait BookingsApi<Ctx = ()>: Send + Sync + 'static {
    type Error: Send;
    /// List all bookings
    fn list_bookings(
        &self,
        ctx: Ctx,
    ) -> impl std::future::Future<Output = Result<ListBookingsResponse, Self::Error>> + Send;
}
/// List all courts
#[derive(Debug)]
pub enum ListCourtsResponse {
    /// A list of courts
    Ok(Vec<super::types::Court>),
    /// Unauthorized
    Unauthorized(super::types::Error),
}
/// Upload a court photo with metadata
#[derive(Debug)]
pub enum UploadCourtPhotoResponse {
    /// Photo uploaded
    NoContent,
    /// Court not found
    NotFound(super::types::Error),
}
#[derive(Debug, Clone)]
pub struct UploadCourtPhotoPath {
    pub court_id: orator_axum::uuid::Uuid,
}
#[derive(Debug)]
pub struct UploadCourtPhotoBody {
    pub caption: Option<String>,
    pub photo: Option<orator_axum::bytes::Bytes>,
}
pub trait CourtsApi<Ctx = ()>: Send + Sync + 'static {
    type Error: Send;
    /// List all courts
    fn list_courts(
        &self,
        ctx: Ctx,
    ) -> impl std::future::Future<Output = Result<ListCourtsResponse, Self::Error>> + Send;
    /// Upload a court photo with metadata
    fn upload_court_photo(
        &self,
        ctx: Ctx,
        path: UploadCourtPhotoPath,
        body: UploadCourtPhotoBody,
    ) -> impl std::future::Future<Output = Result<UploadCourtPhotoResponse, Self::Error>> + Send;
}
/// Check API health
#[derive(Debug)]
pub enum HealthCheckResponse {
    /// Service is healthy
    Ok(String),
}
pub trait HealthApi<Ctx = ()>: Send + Sync + 'static {
    type Error: Send;
    /// Check API health
    fn health_check(
        &self,
        ctx: Ctx,
    ) -> impl std::future::Future<Output = Result<HealthCheckResponse, Self::Error>> + Send;
}
/// List all members
#[derive(Debug)]
pub enum ListMembersResponse {
    /// A list of members
    Ok(Vec<super::types::Member>),
    /// Unauthorized
    Unauthorized(super::types::Error),
    /// Unexpected error
    Default(orator_axum::http::StatusCode, super::types::Error),
}
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(crate = "orator_axum::serde")]
pub struct ListMembersQuery {
    /// Maximum number of members to return
    pub limit: Option<i32>,
    /// Number of members to skip
    pub offset: Option<i32>,
}
#[derive(Debug, Clone)]
pub struct ListMembersHeader {
    /// Unique request identifier for tracing
    pub x_request_id: String,
    /// Optional page size override via header
    pub x_page_size: Option<i32>,
}
#[derive(Debug, Clone)]
pub struct ListMembersCookie {
    /// Session identifier cookie
    pub session_id: String,
}
/// Create a new member
#[derive(Debug)]
pub enum CreateMemberResponse {
    /// The created member
    Created(super::types::Member),
    /// Validation error
    UnprocessableEntity(super::types::Error),
}
/// Get a member by ID
#[derive(Debug)]
pub enum GetMemberResponse {
    /// The member
    Ok(super::types::Member),
    /// Member not found
    NotFound(super::types::Error),
}
#[derive(Debug, Clone)]
pub struct GetMemberPath {
    pub member_id: orator_axum::uuid::Uuid,
}
/// Update a member
#[derive(Debug)]
pub enum UpdateMemberResponse {
    /// The updated member
    Ok(super::types::Member),
    /// Member not found
    NotFound(super::types::Error),
}
#[derive(Debug, Clone)]
pub struct UpdateMemberPath {
    pub member_id: orator_axum::uuid::Uuid,
}
/// Delete a member
#[derive(Debug)]
pub enum DeleteMemberResponse {
    /// Member deleted
    NoContent,
    /// Member not found
    NotFound(super::types::Error),
}
#[derive(Debug, Clone)]
pub struct DeleteMemberPath {
    pub member_id: orator_axum::uuid::Uuid,
}
/// Download a member's profile photo
#[derive(Debug)]
pub enum GetMemberPhotoResponse {
    /// The profile photo
    Ok(orator_axum::bytes::Bytes),
    /// Member or photo not found
    NotFound(super::types::Error),
}
#[derive(Debug, Clone)]
pub struct GetMemberPhotoPath {
    pub member_id: orator_axum::uuid::Uuid,
}
/// Upload a member's profile photo
#[derive(Debug)]
pub enum UploadMemberPhotoResponse {
    /// Photo uploaded
    NoContent,
    /// Member not found
    NotFound(super::types::Error),
}
#[derive(Debug, Clone)]
pub struct UploadMemberPhotoPath {
    pub member_id: orator_axum::uuid::Uuid,
}
pub trait MembersApi<Ctx = ()>: Send + Sync + 'static {
    type Error: Send;
    /// List all members
    fn list_members(
        &self,
        ctx: Ctx,
        query: ListMembersQuery,
        header: ListMembersHeader,
        cookie: ListMembersCookie,
    ) -> impl std::future::Future<Output = Result<ListMembersResponse, Self::Error>> + Send;
    /// Create a new member
    fn create_member(
        &self,
        ctx: Ctx,
        body: super::types::NewMember,
    ) -> impl std::future::Future<Output = Result<CreateMemberResponse, Self::Error>> + Send;
    /// Get a member by ID
    fn get_member(
        &self,
        ctx: Ctx,
        path: GetMemberPath,
    ) -> impl std::future::Future<Output = Result<GetMemberResponse, Self::Error>> + Send;
    /// Update a member
    fn update_member(
        &self,
        ctx: Ctx,
        path: UpdateMemberPath,
        body: super::types::UpdateMember,
    ) -> impl std::future::Future<Output = Result<UpdateMemberResponse, Self::Error>> + Send;
    /// Delete a member
    fn delete_member(
        &self,
        ctx: Ctx,
        path: DeleteMemberPath,
    ) -> impl std::future::Future<Output = Result<DeleteMemberResponse, Self::Error>> + Send;
    /// Download a member's profile photo
    fn get_member_photo(
        &self,
        ctx: Ctx,
        path: GetMemberPhotoPath,
    ) -> impl std::future::Future<Output = Result<GetMemberPhotoResponse, Self::Error>> + Send;
    /// Upload a member's profile photo
    fn upload_member_photo(
        &self,
        ctx: Ctx,
        path: UploadMemberPhotoPath,
        body: orator_axum::bytes::Bytes,
    ) -> impl std::future::Future<Output = Result<UploadMemberPhotoResponse, Self::Error>> + Send;
}
