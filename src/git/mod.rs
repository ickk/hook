//! Provides `routes()` function which returns routes for the `/git` Scope.
//!
//! endpoints:
//! - POST: `/<repo>/pull` - triggers a *git pull* action on `<repo>`.
mod github;
mod config;
mod pull;

use crate::{git::{config::{Config,
                           Service,
                           Action,
                           read_config},
                  github::{Github,
                           Payload,
                           Signature},
                  pull::{do_fetch,
                         do_merge}},
            hmac::hmac_sha256};
use rocket::{Route,
             Data,
             routes,
             post,
             http};
use git2::Repository;
use serde_json;
use std::io::Read;

/// Returns `Vec<Route>` for rocket to *mount*.
pub fn routes() -> Vec<Route> {
  setup();
  routes![pull]
}

lazy_static! {
  #[derive(Debug)]
  static ref CONFIG: Config = read_config();
}

fn setup() {
  eprintln!("Reading config/git.ron...");
  eprintln!("{:#?}", *CONFIG);
}

/// Request handler for *POST* requests to `/<repo>`.
#[post("/<repo_name>", format="json", data="<data>")]
fn pull(repo_name: String,
        _service: Github,
        event: github::Event,
        signature: Signature,
        data: Data,
        ) -> http::Status {

  // Read the body into a string max size of 10KiB.
  let body = {
    let mut data = data.open();
    let mut buffer = [0 as u8; 512];
    let mut body: Vec<u8> = vec![];
    for _ in 0..(10*2) {
      match data.read(&mut buffer) {
        Ok(n) if n==0 => break,
        Ok(n) if n>0 => body.extend(&buffer[0..n]),
        Err(e) => {
          eprintln!("Couldn't read body data: {:#?}", e);
          return http::Status::BadRequest
        },
        _ => unreachable!(),
      }
    }
    match String::from_utf8(body) {
      Ok(s) => s,
      Err(e) => {
        eprintln!("Body data was invalid utf-8: {:#?}", e);
        return http::Status::BadRequest
      }
    }
  };

  // Parse body.as JSON.
  let payload: Payload = match serde_json::from_str(&body) {
    Ok(payload) => payload,
    Err(_) => {
      eprintln!("Invalid structure or form of request body.");
      return http::Status::BadRequest
    },
  };

  // Get the configuration policy that corresponds to the request's secret.
  // If this returns `Some<Policy>` then the secret exists and is valid.
  let policy = match CONFIG.get_policy_from_name(&repo_name) {
    Some(p) => p,
    None => {
      eprintln!("No matching policy.");
      return http::Status::NotFound
    },
  };

  { // Check the service.
    match policy.service {
      Service::Github => (),
      #[allow(unreachable_patterns)]
      _ => {
        eprintln!("Service doesn't match policy.");
        return http::Status::NotFound
      }
    };
  }

  { // Enforce repo_name and payload.repository.name are identical.
    if repo_name != payload.repository.name {
      eprintln!("Endpoint doesn't match policy repository name.");
      return http::Status::NotFound;
    }
  }

  { // Validate the Signature.
    let hmac = hmac_sha256(policy.secret.clone().into(),body.into());
    if hmac != signature {
      eprintln!("HMAC signature is invalid.");
      return http::Status::NotFound
    }
  }

  // Validate and evaluate the event.
  match (&policy.event, event) {
    (config::Event::Push, github::Push) => {
      match policy.action.execute() {
        Ok(_) => http::Status::Ok,
        Err(_) => http::Status::InternalServerError,
      }
    }
    _ => return http::Status::NotFound,
  }
}

impl Action {
  fn execute(&self) -> Result<(),()> {
    match self {
      Action::Pull{path, remote, branch, ssh_key_path} => {
        if let Ok(repo) = Repository::open(path) {
          eprintln!("Opened repository: {:#?}", path);
          let mut remote = match repo.find_remote(remote) {
            Ok(r) => r,
            Err(e) => return {
              eprintln!("Couldn't find remote: {:#?} {:#?}", remote, e);
              Err(())
            },
          };
          let fetch_commit = match do_fetch(&repo,
                                            &[branch],
                                            &mut remote,
                                            &ssh_key_path) {
            Ok(c) => {
              eprintln!("Fetched commit.");
              c
            }
            Err(e) => {
              eprintln!("Error fetching commit: {:#?}", e);
              return Err(())
            },
          };
          eprintln!("Merging commit..");
          match do_merge(&repo, &branch, fetch_commit) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(()),
          }
        }
        Err(())
      }
    }
  }
}

