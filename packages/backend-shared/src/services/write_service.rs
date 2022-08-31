use super::read_service::AbstractReadService;
#[async_trait::async_trait]
pub trait AbstractWriteService: AbstractReadService {}
