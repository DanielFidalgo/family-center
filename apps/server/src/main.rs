mod application;
mod configuration;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<(), configuration::service_setup::ServiceError> {
    configuration::bootstrapper::bootstrap().await
}
