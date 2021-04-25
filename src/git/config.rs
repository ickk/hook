use ron::de::from_reader;
use serde::{Serialize, Deserialize};
use std::{self,
          fs::File,
          env::current_dir,
          collections::HashMap};

pub fn read_config() -> Config {
  let input_path = {
    let mut path = current_dir().unwrap();
    path.push("config");
    path.push("git.ron");
    path
  };

  let f = File::open(&input_path)
               .expect(&format!("Failed to open {:?}", input_path));

  let user_config: UserConfig = if let Ok(c) = from_reader(f) {
    c
  } else {
    panic!("Failed to load config {:?}", input_path);
  };

  Config::from(user_config)
}

impl Config {
  pub fn get_policy_from_name(&self, repo_name: &str) -> Option<&Policy> {
    if let Some(&i) = self.name_map.get(repo_name) {
      return self.policies.get(i)
    }
    None
  }
}

impl From<UserConfig> for Config {
  fn from(user_config: UserConfig) -> Self {
    let policies: Vec<Policy> = user_config.policies.into_iter().map(|p| p.into()).collect();
    // Generate name-to-policy map.
    let mut name_map = HashMap::new();
    for (i, policy) in policies.iter().enumerate() {
      name_map.insert(policy.repo_name.clone(), i);
    }
    if policies.len() != name_map.len() {
      panic!("`repo_name` fields must be unique in config/git.ron.");
    }
    Self {
      policies,
      name_map,
    }
  }
}
impl From<UserPolicy> for Policy {
  fn from(user_policy: UserPolicy) -> Self {
    let names: Vec<&str> = user_policy.full_repo_name.split('/').collect();
    if names.len() != 2 {
      panic!("Invalid `repo_name` in config/git.ron.")
    }
    let (user, repo_name) = (names[0].to_string(), names[1].to_string());

    let mut ssh_url = "git@github.com:".to_string();
    ssh_url.push_str(&user_policy.full_repo_name);
    ssh_url.push_str(".git");

    Self {
      secret: user_policy.secret,
      service: user_policy.service,
      user,
      repo_name,
      full_repo_name: user_policy.full_repo_name,
      ssh_url,
      event: user_policy.event,
      action: user_policy.action,
    }
  }
}

#[derive(Debug)]
pub struct Config {
  pub policies: Vec<Policy>,
  name_map: HashMap<String, usize>,
}
#[derive(Debug)]
pub struct Policy {
  pub secret: String,
  pub service: Service,
  pub user: String,
  pub repo_name: String,
  pub full_repo_name: String,
  pub ssh_url: String,
  pub event: Event,
  pub action: Action,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfig {
  policies: Vec<UserPolicy>,
}
#[derive(Debug, Serialize, Deserialize)]
struct UserPolicy {
  service: Service,
  #[serde(rename = "repo_name")]
  full_repo_name: String,
  secret: String,
  event: Event,
  action: Action,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
  Push
}
#[derive(Debug, Serialize, Deserialize)]
pub enum Service {
  Github
}
#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
  Pull {
    path: String,
    remote: String,
    branch: String,
    ssh_key_path: String,
  }
}
