use backend_core::error::ProblemDetail;

pub trait TestReadRunner {
  type Root;

  type Query;

  type Sort;

  async fn find_all(
    query: Option<Self::Query>,
    limit: Option<u64>,
    sort: Option<Self::Sort>,
  ) -> Result<Self::Root, impl ProblemDetail>;
}
