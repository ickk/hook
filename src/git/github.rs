//! Handle github specific behaviour.

use serde::{Serialize,
            Deserialize};
use rocket::{Outcome,
             Request,
             request::{self,
                       FromRequest},
             http::Status};
use hex;

#[derive(Debug)]
pub enum SignatureError {
  Missing,
}

/// Contains the `X-Hub-Signature-256` request header value.
#[derive(Debug, PartialEq)]
pub struct Signature(String);

/// Allow comparison of `Signature` with `Vec<u8>`.
impl PartialEq<Vec<u8>> for Signature {
  fn eq(&self, other: &Vec<u8>) -> bool {
    let mut cmp = "sha256=".to_string(); cmp.push_str(&hex::encode(other));
    self.0 == cmp
  }
}
/// Allow comparison of `Vec<u8>` with `Signature`.
impl PartialEq<Signature> for Vec<u8> {
  fn eq(&self, other: &Signature) -> bool {
    let mut cmp = "sha256=".to_string(); cmp.push_str(&hex::encode(self));
    cmp == other.0
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for Signature {
  type Error = SignatureError;

  fn from_request(request: &'a Request<'r>)
  -> request::Outcome<Self, Self::Error> {
    if let Some(sig) = request.headers().get_one("X-Hub-Signature-256") {
      Outcome::Success(Self(sig.to_string()))
    } else {
      Outcome::Failure((Status::NotFound, Self::Error::Missing))
    }
  }
}

/// Enum representing the `X-GitHub-Event` header value.
pub enum Event {
  Push,
  #[allow(dead_code)]
  Issue,
}
pub use Event::Push;

#[derive(Debug)]
pub enum EventError {
  Missing,
  Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for Event {
  type Error = EventError;

  fn from_request(request: &'a Request<'r>)
  -> request::Outcome<Self, Self::Error> {
    match request.headers().get_one("X-GitHub-Event") {
      Some("push") => Outcome::Success(Self::Push),
      Some(_) => Outcome::Failure((Status::NotFound, Self::Error::Invalid)),
      None => Outcome::Failure((Status::NotFound, Self::Error::Missing)),
    }
  }
}

/// Struct representing the Github User-Agent.
pub struct Github;

#[derive(Debug)]
pub enum ServiceError{
  Missing,
  Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for Github {
  type Error = ServiceError;

  fn from_request(request: &'a Request<'r>)
  -> request::Outcome<Self, Self::Error> {
    match request.headers().get_one("User-Agent") {
      Some(user_agent) if user_agent.starts_with("GitHub-Hookshot/")
        => Outcome::Success(Self),
      Some(_) => Outcome::Failure((Status::NotFound, Self::Error::Invalid)),
      None => Outcome::Failure((Status::NotFound, Self::Error::Missing)),
    }
  }
}

/// `Payload` has the following structure:
///
/// ```
/// Payload {
///   repository: {
///     id: u32,
///     name: String,
///     private: bool,
///     owner: {
///       login: String,
///       id: u32,
///     },
///     html_url: String,
///     ssh_url: String,
///   },
/// }
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
  pub repository: PayloadRepository,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PayloadRepository {
  pub id: u32,
  pub name: String,
  pub full_name: String,
  pub private: bool,
  pub owner: PayloadRepositoryOwner,
  pub html_url: String,
  pub ssh_url: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PayloadRepositoryOwner {
  pub login: String,
  pub id: u32,
}
