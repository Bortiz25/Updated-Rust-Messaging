use warp::reject;

#[derive(Debug)]
pub struct DatabaseError {}
impl reject::Reject for DatabaseError {}

#[derive(Debug)]
pub struct ConflictError {}
impl reject::Reject for ConflictError {}
