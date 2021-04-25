use chrono;

fn main() {
  println!("cargo:rustc-env=BUILD_DATE={}", chrono::Local::now().to_rfc2822());
}
