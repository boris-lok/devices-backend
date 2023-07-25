#[derive(Debug)]
pub enum Permission {
    Role(String),
    IndividualPermission(Vec<String>),
    Empty,
}
