use rocket::{Outcome, Request};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome as RequestOutcome};

use crate::AUTHORIZATION;

pub struct AuthorizationGuard;

impl<'a, 'r> FromRequest<'a, 'r> for AuthorizationGuard {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> RequestOutcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            None => Outcome::Failure((Status::Unauthorized, ())),
            Some(auth) if &*AUTHORIZATION != auth => Outcome::Failure((Status::Forbidden, ())),
            Some(_) => Outcome::Success(Self {})
        }
    }
}