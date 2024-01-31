use async_trait::async_trait;

use crate::you::You;

pub mod just_printer;
pub mod total_printer;

#[async_trait]
pub trait Printer: Sync + Send {
    async fn print(&self, you: &You);
}
