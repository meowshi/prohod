use async_trait::async_trait;
use chrono::Duration;
use cli_table::{print_stdout, WithTitle};

use crate::{session_info::SessionInfo, you::You};

use super::Printer;

pub struct TotalPrinter;
#[async_trait]
impl Printer for TotalPrinter {
    async fn print(&self, you: &You) {
        let sessions = you.get_sessions().await;

        let mut sessions_info = vec![];

        let mut total = SessionInfo::default();

        let mut time = you.from;

        while time <= you.to {
            let mut info = you.get_session_info(sessions.get(&time).unwrap()).await;
            info.time = time.format("%H:%M").to_string();

            total += &info;

            sessions_info.push(info);

            time += Duration::minutes(30);
        }

        total.time = "Итого".to_owned();
        sessions_info.push(total);

        println!(
            "\n          ЦИФРЫ НА {}",
            chrono::Local::now().format("%H:%M:%S")
        );
        print_stdout(sessions_info.with_title()).unwrap();
    }
}
