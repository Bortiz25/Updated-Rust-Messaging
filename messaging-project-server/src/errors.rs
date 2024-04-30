use warp::reject;

#[derive(Debug)]
pub struct DatabaseError {}
impl reject::Reject for DatabaseError {}

#[derive(Debug)]
pub struct ConflictError {}
impl reject::Reject for ConflictError {}

#[derive(Debug)]
pub struct AuthorizationError {}
impl reject::Reject for AuthorizationError {}

#[derive(Debug)]
pub struct JwtError;
impl reject::Reject for JwtError {}
