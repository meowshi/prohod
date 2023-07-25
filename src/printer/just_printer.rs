use async_trait::async_trait;
use chrono::Duration;
use cli_table::{print_stdout, WithTitle};

use crate::you::You;

use super::Printer;

pub struct JustPrinter;
#[async_trait]
impl Printer for JustPrinter {
    async fn print(&self, you: &You) {
        let sessions = you.get_sessions().await;

        let mut times = sessions.keys().collect::<Vec<_>>();
        times.sort();

        let mut sessions_info = vec![];

        let mut time = you.from;

        while time <= you.to {
            println!("{time}");
            let mut info = you.get_session_info(sessions.get(&time).unwrap()).await;
            info.time = time.format("%H:%M").to_string();

            sessions_info.push(info);

            time += Duration::minutes(30);
        }
        println!(
            "\n          ЦИФРЫ НА {}",
            chrono::Local::now().format("%H:%M:%S")
        );
        print_stdout(sessions_info.with_title()).unwrap();
    }
}
