use tonic::metadata::MetadataValue;
use tonic::{Request, Status};

pub(crate) fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
  let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();
  log::info!("Metadata: {:#?}", req.metadata());

  match req.metadata().get("authorization") {
    Some(t) if token == t => Ok(req),
    _ => Err(Status::unauthenticated("No valid auth token")),
  }
}
