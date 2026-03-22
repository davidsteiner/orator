#![allow(dead_code, unused_imports)]
impl orator_axum::axum::response::IntoResponse for super::operations::ListBookingsResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Ok(body) => (
                orator_axum::http::StatusCode::OK,
                orator_axum::axum::Json(body),
            )
                .into_response(),
            Self::Unauthorized(body) => (
                orator_axum::http::StatusCode::UNAUTHORIZED,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_list_bookings<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
) -> Result<super::operations::ListBookingsResponse, T::Error>
where
    T: super::operations::BookingsApi<Ctx>,
{
    api.list_bookings(ctx).await
}
pub fn bookings_router<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
where
    T: super::operations::BookingsApi<Ctx>,
    T::Error: orator_axum::axum::response::IntoResponse,
    Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
{
    orator_axum::axum::Router::new()
        .route(
            "/bookings",
            orator_axum::axum::routing::get(handle_list_bookings::<T, Ctx>),
        )
        .with_state(api)
}
impl orator_axum::axum::response::IntoResponse for super::operations::ListCourtsResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Ok(body) => (
                orator_axum::http::StatusCode::OK,
                orator_axum::axum::Json(body),
            )
                .into_response(),
            Self::Unauthorized(body) => (
                orator_axum::http::StatusCode::UNAUTHORIZED,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_list_courts<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
) -> Result<super::operations::ListCourtsResponse, T::Error>
where
    T: super::operations::CourtsApi<Ctx>,
{
    api.list_courts(ctx).await
}
impl orator_axum::axum::response::IntoResponse for super::operations::UploadCourtPhotoResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::NoContent => orator_axum::http::StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(body) => (
                orator_axum::http::StatusCode::NOT_FOUND,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
impl<S> orator_axum::axum::extract::FromRequest<S> for super::operations::UploadCourtPhotoBody
where
    S: Send + Sync,
{
    type Rejection = orator_axum::axum::response::Response;
    async fn from_request(
        req: orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut multipart = orator_axum::axum::extract::Multipart::from_request(req, state)
            .await
            .map_err(|e| {
                use orator_axum::axum::response::IntoResponse;
                e.into_response()
            })?;
        let mut caption = None;
        let mut photo = None;
        while let Some(field) = multipart.next_field().await.map_err(|e| {
            orator_axum::axum::response::Response::builder()
                .status(orator_axum::http::StatusCode::BAD_REQUEST)
                .body(orator_axum::axum::body::Body::from(e.to_string()))
                .unwrap()
        })? {
            let name = field.name().unwrap_or_default().to_owned();
            match name.as_str() {
                "caption" => {
                    caption = Some(field.text().await.map_err(|e| {
                        orator_axum::axum::response::Response::builder()
                            .status(orator_axum::http::StatusCode::BAD_REQUEST)
                            .body(orator_axum::axum::body::Body::from(e.to_string()))
                            .unwrap()
                    })?);
                }
                "photo" => {
                    photo = Some(field.bytes().await.map_err(|e| {
                        orator_axum::axum::response::Response::builder()
                            .status(orator_axum::http::StatusCode::BAD_REQUEST)
                            .body(orator_axum::axum::body::Body::from(e.to_string()))
                            .unwrap()
                    })?);
                }
                _ => {}
            }
        }
        Ok(Self { caption, photo })
    }
}
async fn handle_upload_court_photo<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Path(court_id): orator_axum::axum::extract::Path<
        orator_axum::uuid::Uuid,
    >,
    body: super::operations::UploadCourtPhotoBody,
) -> Result<super::operations::UploadCourtPhotoResponse, T::Error>
where
    T: super::operations::CourtsApi<Ctx>,
{
    api.upload_court_photo(
        ctx,
        super::operations::UploadCourtPhotoPath { court_id },
        body,
    )
    .await
}
pub fn courts_router<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
where
    T: super::operations::CourtsApi<Ctx>,
    T::Error: orator_axum::axum::response::IntoResponse,
    Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
{
    orator_axum::axum::Router::new()
        .route(
            "/courts",
            orator_axum::axum::routing::get(handle_list_courts::<T, Ctx>),
        )
        .route(
            "/courts/{court_id}/photo",
            orator_axum::axum::routing::put(handle_upload_court_photo::<T, Ctx>),
        )
        .with_state(api)
}
impl orator_axum::axum::response::IntoResponse for super::operations::HealthCheckResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Ok(body) => (orator_axum::http::StatusCode::OK, body).into_response(),
        }
    }
}
async fn handle_health_check<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
) -> Result<super::operations::HealthCheckResponse, T::Error>
where
    T: super::operations::HealthApi<Ctx>,
{
    api.health_check(ctx).await
}
pub fn health_router<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
where
    T: super::operations::HealthApi<Ctx>,
    T::Error: orator_axum::axum::response::IntoResponse,
    Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
{
    orator_axum::axum::Router::new()
        .route(
            "/health",
            orator_axum::axum::routing::get(handle_health_check::<T, Ctx>),
        )
        .with_state(api)
}
impl orator_axum::axum::response::IntoResponse for super::operations::ListMembersResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Ok(body) => (
                orator_axum::http::StatusCode::OK,
                orator_axum::axum::Json(body),
            )
                .into_response(),
            Self::Unauthorized(body) => (
                orator_axum::http::StatusCode::UNAUTHORIZED,
                orator_axum::axum::Json(body),
            )
                .into_response(),
            Self::Default(status, body) => (status, orator_axum::axum::Json(body)).into_response(),
        }
    }
}
impl<S> orator_axum::axum::extract::FromRequestParts<S> for super::operations::ListMembersHeader
where
    S: Send + Sync,
{
    type Rejection = orator_axum::ParamRejection;
    async fn from_request_parts(
        parts: &mut orator_axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        Ok(Self {
            x_request_id: headers
                .get("X-Request-ID")
                .ok_or_else(|| {
                    orator_axum::ParamRejection::missing(
                        orator_axum::ParamLocation::Header,
                        "X-Request-ID",
                    )
                })?
                .to_str()
                .map_err(|_| {
                    orator_axum::ParamRejection::non_ascii(
                        orator_axum::ParamLocation::Header,
                        "X-Request-ID",
                    )
                })?
                .to_owned(),
            x_page_size: match headers.get("X-Page-Size") {
                Some(v) => Some(
                    v.to_str()
                        .map_err(|_| {
                            orator_axum::ParamRejection::non_ascii(
                                orator_axum::ParamLocation::Header,
                                "X-Page-Size",
                            )
                        })?
                        .parse::<i32>()
                        .map_err(|e| {
                            orator_axum::ParamRejection::invalid(
                                orator_axum::ParamLocation::Header,
                                "X-Page-Size",
                                e,
                            )
                        })?,
                ),
                None => None,
            },
        })
    }
}
impl<S> orator_axum::axum::extract::FromRequestParts<S> for super::operations::ListMembersCookie
where
    S: Send + Sync,
{
    type Rejection = orator_axum::ParamRejection;
    async fn from_request_parts(
        parts: &mut orator_axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let jar = orator_axum::axum_extra::extract::CookieJar::from_request_parts(parts, state)
            .await
            .unwrap();
        Ok(Self {
            session_id: jar
                .get("session_id")
                .ok_or_else(|| {
                    orator_axum::ParamRejection::missing(
                        orator_axum::ParamLocation::Cookie,
                        "session_id",
                    )
                })?
                .value()
                .to_owned(),
        })
    }
}
async fn handle_list_members<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Query(query): orator_axum::axum::extract::Query<
        super::operations::ListMembersQuery,
    >,
    header: super::operations::ListMembersHeader,
    cookie: super::operations::ListMembersCookie,
) -> Result<super::operations::ListMembersResponse, T::Error>
where
    T: super::operations::MembersApi<Ctx>,
{
    api.list_members(ctx, query, header, cookie).await
}
impl orator_axum::axum::response::IntoResponse for super::operations::CreateMemberResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Created(body) => (
                orator_axum::http::StatusCode::CREATED,
                orator_axum::axum::Json(body),
            )
                .into_response(),
            Self::UnprocessableEntity(body) => (
                orator_axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_create_member<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::Json(body): orator_axum::axum::Json<super::types::NewMember>,
) -> Result<super::operations::CreateMemberResponse, T::Error>
where
    T: super::operations::MembersApi<Ctx>,
{
    api.create_member(ctx, body).await
}
impl orator_axum::axum::response::IntoResponse for super::operations::GetMemberResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Ok(body) => (
                orator_axum::http::StatusCode::OK,
                orator_axum::axum::Json(body),
            )
                .into_response(),
            Self::NotFound(body) => (
                orator_axum::http::StatusCode::NOT_FOUND,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_get_member<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<
        orator_axum::uuid::Uuid,
    >,
) -> Result<super::operations::GetMemberResponse, T::Error>
where
    T: super::operations::MembersApi<Ctx>,
{
    api.get_member(ctx, super::operations::GetMemberPath { member_id })
        .await
}
impl orator_axum::axum::response::IntoResponse for super::operations::UpdateMemberResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Ok(body) => (
                orator_axum::http::StatusCode::OK,
                orator_axum::axum::Json(body),
            )
                .into_response(),
            Self::NotFound(body) => (
                orator_axum::http::StatusCode::NOT_FOUND,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_update_member<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<
        orator_axum::uuid::Uuid,
    >,
    orator_axum::axum::Json(body): orator_axum::axum::Json<super::types::UpdateMember>,
) -> Result<super::operations::UpdateMemberResponse, T::Error>
where
    T: super::operations::MembersApi<Ctx>,
{
    api.update_member(ctx, super::operations::UpdateMemberPath { member_id }, body)
        .await
}
impl orator_axum::axum::response::IntoResponse for super::operations::DeleteMemberResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::NoContent => orator_axum::http::StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(body) => (
                orator_axum::http::StatusCode::NOT_FOUND,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_delete_member<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<
        orator_axum::uuid::Uuid,
    >,
) -> Result<super::operations::DeleteMemberResponse, T::Error>
where
    T: super::operations::MembersApi<Ctx>,
{
    api.delete_member(ctx, super::operations::DeleteMemberPath { member_id })
        .await
}
impl orator_axum::axum::response::IntoResponse for super::operations::GetMemberPhotoResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::Ok(body) => (orator_axum::http::StatusCode::OK, body).into_response(),
            Self::NotFound(body) => (
                orator_axum::http::StatusCode::NOT_FOUND,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_get_member_photo<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<
        orator_axum::uuid::Uuid,
    >,
) -> Result<super::operations::GetMemberPhotoResponse, T::Error>
where
    T: super::operations::MembersApi<Ctx>,
{
    api.get_member_photo(ctx, super::operations::GetMemberPhotoPath { member_id })
        .await
}
impl orator_axum::axum::response::IntoResponse for super::operations::UploadMemberPhotoResponse {
    fn into_response(self) -> orator_axum::axum::response::Response {
        match self {
            Self::NoContent => orator_axum::http::StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(body) => (
                orator_axum::http::StatusCode::NOT_FOUND,
                orator_axum::axum::Json(body),
            )
                .into_response(),
        }
    }
}
async fn handle_upload_member_photo<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<
        orator_axum::uuid::Uuid,
    >,
    body: orator_axum::axum::body::Bytes,
) -> Result<super::operations::UploadMemberPhotoResponse, T::Error>
where
    T: super::operations::MembersApi<Ctx>,
{
    api.upload_member_photo(
        ctx,
        super::operations::UploadMemberPhotoPath { member_id },
        body,
    )
    .await
}
pub fn members_router<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
where
    T: super::operations::MembersApi<Ctx>,
    T::Error: orator_axum::axum::response::IntoResponse,
    Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
{
    orator_axum::axum::Router::new()
        .route(
            "/members",
            orator_axum::axum::routing::get(handle_list_members::<T, Ctx>)
                .post(handle_create_member::<T, Ctx>),
        )
        .route(
            "/members/{member_id}",
            orator_axum::axum::routing::get(handle_get_member::<T, Ctx>)
                .patch(handle_update_member::<T, Ctx>)
                .delete(handle_delete_member::<T, Ctx>),
        )
        .route(
            "/members/{member_id}/photo",
            orator_axum::axum::routing::get(handle_get_member_photo::<T, Ctx>)
                .put(handle_upload_member_photo::<T, Ctx>),
        )
        .with_state(api)
}
pub struct Missing;
pub struct Registered;
pub struct BookingsRouter(orator_axum::axum::Router);
impl BookingsRouter {
    pub fn new<T, Ctx>(api: std::sync::Arc<T>) -> Self
    where
        T: super::operations::BookingsApi<Ctx>,
        T::Error: orator_axum::axum::response::IntoResponse,
        Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
    {
        Self(bookings_router(api))
    }
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: orator_axum::tower::Layer<orator_axum::axum::routing::Route>
            + Clone
            + Send
            + Sync
            + 'static,
        L::Service: orator_axum::tower::Service<
                orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
            > + Clone
            + Send
            + Sync
            + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Response: orator_axum::axum::response::IntoResponse + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Error: Into<std::convert::Infallible> + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Future: Send + 'static,
    {
        Self(self.0.layer(layer))
    }
}
pub struct CourtsRouter(orator_axum::axum::Router);
impl CourtsRouter {
    pub fn new<T, Ctx>(api: std::sync::Arc<T>) -> Self
    where
        T: super::operations::CourtsApi<Ctx>,
        T::Error: orator_axum::axum::response::IntoResponse,
        Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
    {
        Self(courts_router(api))
    }
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: orator_axum::tower::Layer<orator_axum::axum::routing::Route>
            + Clone
            + Send
            + Sync
            + 'static,
        L::Service: orator_axum::tower::Service<
                orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
            > + Clone
            + Send
            + Sync
            + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Response: orator_axum::axum::response::IntoResponse + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Error: Into<std::convert::Infallible> + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Future: Send + 'static,
    {
        Self(self.0.layer(layer))
    }
}
pub struct HealthRouter(orator_axum::axum::Router);
impl HealthRouter {
    pub fn new<T, Ctx>(api: std::sync::Arc<T>) -> Self
    where
        T: super::operations::HealthApi<Ctx>,
        T::Error: orator_axum::axum::response::IntoResponse,
        Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
    {
        Self(health_router(api))
    }
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: orator_axum::tower::Layer<orator_axum::axum::routing::Route>
            + Clone
            + Send
            + Sync
            + 'static,
        L::Service: orator_axum::tower::Service<
                orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
            > + Clone
            + Send
            + Sync
            + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Response: orator_axum::axum::response::IntoResponse + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Error: Into<std::convert::Infallible> + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Future: Send + 'static,
    {
        Self(self.0.layer(layer))
    }
}
pub struct MembersRouter(orator_axum::axum::Router);
impl MembersRouter {
    pub fn new<T, Ctx>(api: std::sync::Arc<T>) -> Self
    where
        T: super::operations::MembersApi<Ctx>,
        T::Error: orator_axum::axum::response::IntoResponse,
        Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
    {
        Self(members_router(api))
    }
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: orator_axum::tower::Layer<orator_axum::axum::routing::Route>
            + Clone
            + Send
            + Sync
            + 'static,
        L::Service: orator_axum::tower::Service<
                orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
            > + Clone
            + Send
            + Sync
            + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Response: orator_axum::axum::response::IntoResponse + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Error: Into<std::convert::Infallible> + 'static,
        <L::Service as orator_axum::tower::Service<
            orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
        >>::Future: Send + 'static,
    {
        Self(self.0.layer(layer))
    }
}
pub struct ApiBuilder<
    BookingsState = Missing,
    CourtsState = Missing,
    HealthState = Missing,
    MembersState = Missing,
> {
    router: orator_axum::axum::Router,
    _phantom: std::marker::PhantomData<(BookingsState, CourtsState, HealthState, MembersState)>,
}
impl ApiBuilder {
    pub fn new() -> Self {
        Self {
            router: orator_axum::axum::Router::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<CourtsState, HealthState, MembersState>
    ApiBuilder<Missing, CourtsState, HealthState, MembersState>
{
    pub fn bookings(
        self,
        router: BookingsRouter,
    ) -> ApiBuilder<Registered, CourtsState, HealthState, MembersState> {
        ApiBuilder {
            router: self.router.merge(router.0),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<BookingsState, HealthState, MembersState>
    ApiBuilder<BookingsState, Missing, HealthState, MembersState>
{
    pub fn courts(
        self,
        router: CourtsRouter,
    ) -> ApiBuilder<BookingsState, Registered, HealthState, MembersState> {
        ApiBuilder {
            router: self.router.merge(router.0),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<BookingsState, CourtsState, MembersState>
    ApiBuilder<BookingsState, CourtsState, Missing, MembersState>
{
    pub fn health(
        self,
        router: HealthRouter,
    ) -> ApiBuilder<BookingsState, CourtsState, Registered, MembersState> {
        ApiBuilder {
            router: self.router.merge(router.0),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<BookingsState, CourtsState, HealthState>
    ApiBuilder<BookingsState, CourtsState, HealthState, Missing>
{
    pub fn members(
        self,
        router: MembersRouter,
    ) -> ApiBuilder<BookingsState, CourtsState, HealthState, Registered> {
        ApiBuilder {
            router: self.router.merge(router.0),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl ApiBuilder<Registered, Registered, Registered, Registered> {
    pub fn build(self) -> orator_axum::axum::Router {
        self.router
    }
}
