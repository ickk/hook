//! A web server designed to perform actions in response to webhook callbacks.
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate lazy_static;

use rocket;

mod config;
mod git;
mod hmac;

lazy_static! {
  #[derive(Debug)]
  static ref CONFIG: config::Config = config::read_config();
}

/// Configurable value used to correct for a reverse proxy serving the Hook
/// application under a path.
#[allow(dead_code)]
pub(crate) fn prepend_url_base(mut url: String) -> String {
  url.insert_str(0, &CONFIG.url_base);
  url
}

fn setup() {
  eprintln!("Reading config/hook.ron...");
  eprintln!("{:#?}", *CONFIG);
}

fn main() {
  if let Some(build_date) = option_env!("BUILD_DATE") {
    eprintln!("Build date: {}", build_date)
  }
  setup();

  // Set up Rocket using values defined by `config/hook.ron`.
  let rconf = rocket::config::Config::build(rocket::config::Environment::Staging)
    .address(&CONFIG.address)
    .port(CONFIG.port)
    .finalize()
    .unwrap();

  // Attach modules as defined by `config/hook.ron`.
  let mut r = rocket::custom(rconf);
  for module in CONFIG.modules.iter() {
    match module {
      config::Module::Git{mount_path} => {
        r = r.mount(mount_path, git::routes());
      }
    }
  }
  r.launch();
}
