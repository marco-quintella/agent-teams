mod ingestor;
mod schema;

pub use ingestor::AtopIngestor;
pub use schema::AtopMessage;

pub const ATOP_V1_SPEC: &str = include_str!("../../resources/atop-v1.md");
