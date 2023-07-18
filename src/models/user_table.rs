#[derive(Debug, sea_query::Iden)]
pub enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
}
