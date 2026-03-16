use super::operations::*;
use super::types::*;

impl orator_axum::axum::response::IntoResponse for ListBookingsResponse {
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
) -> Result<ListBookingsResponse, T::Error>
where
    T: BookingsApi<Ctx>,
{
    api.list_bookings(ctx).await
}
pub fn bookings_router<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
where
    T: BookingsApi<Ctx>,
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
impl orator_axum::axum::response::IntoResponse for ListCourtsResponse {
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
) -> Result<ListCourtsResponse, T::Error>
where
    T: CourtsApi<Ctx>,
{
    api.list_courts(ctx).await
}
pub fn courts_router<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
where
    T: CourtsApi<Ctx>,
    T::Error: orator_axum::axum::response::IntoResponse,
    Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
{
    orator_axum::axum::Router::new()
        .route(
            "/courts",
            orator_axum::axum::routing::get(handle_list_courts::<T, Ctx>),
        )
        .with_state(api)
}
impl orator_axum::axum::response::IntoResponse for ListMembersResponse {
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
async fn handle_list_members<T, Ctx>(
    orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>,
    ctx: Ctx,
    orator_axum::axum::extract::Query(query): orator_axum::axum::extract::Query<ListMembersQuery>,
    headers: orator_axum::axum::http::HeaderMap,
    jar: orator_axum::axum_extra::extract::CookieJar,
) -> Result<ListMembersResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    let header = ListMembersHeader {
        x_request_id: headers
            .get("X-Request-ID")
            .expect(concat!("missing required header: ", "X-Request-ID"))
            .to_str()
            .expect(concat!("non-ASCII header value: ", "X-Request-ID"))
            .to_owned(),
        x_page_size: headers.get("X-Page-Size").map(|v| {
            v.to_str()
                .expect(concat!("non-ASCII header value: ", "X-Page-Size"))
                .parse::<i32>()
                .expect(concat!("invalid header value: ", "X-Page-Size"))
        }),
    };
    let cookie = ListMembersCookie {
        session_id: jar
            .get("session_id")
            .expect(concat!("missing required cookie: ", "session_id"))
            .value()
            .to_owned(),
    };
    api.list_members(ctx, query, header, cookie).await
}
impl orator_axum::axum::response::IntoResponse for CreateMemberResponse {
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
    orator_axum::axum::Json(body): orator_axum::axum::Json<NewMember>,
) -> Result<CreateMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.create_member(ctx, body).await
}
impl orator_axum::axum::response::IntoResponse for GetMemberResponse {
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
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<i64>,
) -> Result<GetMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.get_member(ctx, GetMemberPath { member_id }).await
}
impl orator_axum::axum::response::IntoResponse for UpdateMemberResponse {
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
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<i64>,
    orator_axum::axum::Json(body): orator_axum::axum::Json<UpdateMember>,
) -> Result<UpdateMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.update_member(ctx, UpdateMemberPath { member_id }, body)
        .await
}
impl orator_axum::axum::response::IntoResponse for DeleteMemberResponse {
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
    orator_axum::axum::extract::Path(member_id): orator_axum::axum::extract::Path<i64>,
) -> Result<DeleteMemberResponse, T::Error>
where
    T: MembersApi<Ctx>,
{
    api.delete_member(ctx, DeleteMemberPath { member_id }).await
}
pub fn members_router<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
where
    T: MembersApi<Ctx>,
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
        .with_state(api)
}
pub struct Missing;
pub struct Registered;
pub struct BookingsRouter(orator_axum::axum::Router);
impl BookingsRouter {
    pub fn new<T, Ctx>(api: std::sync::Arc<T>) -> Self
    where
        T: BookingsApi<Ctx>,
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
        T: CourtsApi<Ctx>,
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
pub struct MembersRouter(orator_axum::axum::Router);
impl MembersRouter {
    pub fn new<T, Ctx>(api: std::sync::Arc<T>) -> Self
    where
        T: MembersApi<Ctx>,
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
pub struct ApiBuilder<BookingsState = Missing, CourtsState = Missing, MembersState = Missing> {
    router: orator_axum::axum::Router,
    _phantom: std::marker::PhantomData<(BookingsState, CourtsState, MembersState)>,
}
impl ApiBuilder {
    pub fn new() -> Self {
        Self {
            router: orator_axum::axum::Router::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<CourtsState, MembersState> ApiBuilder<Missing, CourtsState, MembersState> {
    pub fn bookings(
        self,
        router: BookingsRouter,
    ) -> ApiBuilder<Registered, CourtsState, MembersState> {
        ApiBuilder {
            router: self.router.merge(router.0),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<BookingsState, MembersState> ApiBuilder<BookingsState, Missing, MembersState> {
    pub fn courts(
        self,
        router: CourtsRouter,
    ) -> ApiBuilder<BookingsState, Registered, MembersState> {
        ApiBuilder {
            router: self.router.merge(router.0),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<BookingsState, CourtsState> ApiBuilder<BookingsState, CourtsState, Missing> {
    pub fn members(
        self,
        router: MembersRouter,
    ) -> ApiBuilder<BookingsState, CourtsState, Registered> {
        ApiBuilder {
            router: self.router.merge(router.0),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl ApiBuilder<Registered, Registered, Registered> {
    pub fn build(self) -> orator_axum::axum::Router {
        self.router
    }
}
