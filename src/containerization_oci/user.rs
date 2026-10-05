/// `User`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct User {
  pub uid: u32,
  pub gid: u32,
  pub umask: Option<u32>,
  pub additional_gids: Vec<u32>,
  pub username: String,
}
