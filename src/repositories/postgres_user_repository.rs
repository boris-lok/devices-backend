use anyhow::Context;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use secrecy::Secret;
use sqlx::Row;

use crate::{errors::AppError, models::user_table::Users, utils::PostgresSession};

use super::i_user_repository::IUserRespository;

pub struct PostgresUserRepository {
    session: PostgresSession,
}

impl PostgresUserRepository {
    pub fn new(session: PostgresSession) -> Self {
        Self { session }
    }
}

#[async_trait::async_trait]
impl IUserRespository for PostgresUserRepository {
    async fn get_store_credentials(
        &self,
        username: &str,
    ) -> anyhow::Result<Option<(uuid::Uuid, Secret<String>)>> {
        let mut conn = self.session.get_session().await;

        let sql = Query::select()
            .columns([Users::Id, Users::PasswordHash])
            .from(Users::Table)
            .and_where(Expr::col(Users::Username).eq(username))
            .to_string(PostgresQueryBuilder);

        let res = sqlx::query(&sql)
            //.fetch_optional(conn.deref_mut())
            // In 0.7, `Transaction` can no longer implement `Executor` directly,
            // so it must be dereferences to the internal connection type
            .fetch_optional(&mut **conn)
            .await
            .context("Failed to perform a sql to retrieve stored credentials")
            .map_err(AppError::UnexpectedError)?
            .map(|e| {
                let id = e.get::<uuid::Uuid, usize>(0);
                let password = e.get::<String, usize>(1);
                (id, Secret::new(password))
            });

        Ok(res)
    }
}
