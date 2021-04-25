use ron::de::from_reader;
use serde::{Serialize, Deserialize};
use std::{self,
          fs::File,
          env::current_dir};

pub fn read_config() -> Config {
  let input_path = {
    let mut path = current_dir().unwrap();
    path.push("config");
    path.push("hook.ron");
    path
  };

  let f = File::open(&input_path)
               .expect(&format!("Failed to open {:?}", input_path));

  if let Ok(config) = from_reader(f) {
    config
  } else {
    eprintln!("Failed to load config {:?}", input_path);
    std::process::exit(1);
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  #[serde(default = "default_config_address")]
  pub address: String,
  #[serde(default = "default_config_port")]
  pub port: u16,
  #[serde(default = "default_config_url_base")]
  pub url_base: String,
  #[serde(default = "default_config_modules")]
  pub modules: Vec<Module>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Module {
  Git {
    #[serde(default = "default_config_mount_path")]
    mount_path: String,
  },
}

// This is really silly.
// Upstream Issue: https://github.com/serde-rs/serde/issues/368
// PR review: https://github.com/serde-rs/serde/pull/1490#pullrequestreview-288464564
fn default_config_address() -> String { "localhost".into() }
fn default_config_port() -> u16 { 7267 }
fn default_config_url_base() -> String { "".into() }
fn default_config_modules<T>() -> Vec<T> { vec![] }
fn default_config_mount_path() -> String { "/git".into() }
