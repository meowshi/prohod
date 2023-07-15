use std::env;

use chrono::{Datelike, Local, NaiveDate, NaiveTime, Weekday};
use clap::Parser;
use prohod::{Day, You};

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let cookie = env::var("COOKIE").expect("Не установлена переменная окружения COOKIE");
    let login = env::var("LOGIN").expect("Не установлена переменная окружения LOGIN");
    let password = env::var("PASSWORD").expect("Не установлена переменная окружения PASSWORD");

    let mut args = Args::parse();
    args.polish();

    let day = match args.weekend {
        Some(_) => Day::Weekend,
        None => define_day(Local::now().date_naive()),
    };

    let me = You::new(
        &args.date.unwrap(),
        args.from.unwrap(),
        args.to.unwrap(),
        &cookie,
        &login,
        day,
        &password,
        args.total,
    );

    me.print_sessions().await;
}

fn parse_time(time: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(time, "%H:%M")
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    date: Option<String>,

    #[arg(short, long, value_parser = parse_time)]
    from: Option<NaiveTime>,

    #[arg(short, long, value_parser = parse_time)]
    to: Option<NaiveTime>,

    #[arg(short, long)]
    weekend: Option<bool>,

    #[arg(long, default_value = "false")]
    total: bool,
}

impl Args {
    fn polish(&mut self) {
        let today = Local::now().date_naive();

        if self.date.is_none() {
            self.date = Some(Local::now().date_naive().format("%d.%m.%Y").to_string());
        }

        if self.from.is_none() {
            self.from = match today.weekday() {
                Weekday::Sat | Weekday::Sun => NaiveTime::from_hms_opt(11, 0, 0),
                _ => NaiveTime::from_hms_opt(12, 0, 0),
            };
        }

        if self.to.is_none() {
            self.to = NaiveTime::from_hms_opt(21, 0, 0)
        }
    }
}

fn define_day(date: NaiveDate) -> Day {
    match date.weekday() {
        Weekday::Sat | Weekday::Sun => Day::Weekend,
        _ => Day::Weekend,
    }
}
