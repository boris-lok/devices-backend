use secrecy::Secret;

#[async_trait::async_trait]
pub trait IUserRespository {
    async fn get_store_credentials(
        &self,
        username: &str,
    ) -> anyhow::Result<Option<(uuid::Uuid, Secret<String>)>>;
}
