use std::{collections::HashMap, fmt::Display, ops::AddAssign};

use async_trait::async_trait;
use chrono::{Duration, NaiveTime};
use cli_table::{format::Justify, print_stdout, Table, WithTitle};
use md5::{Digest, Md5};
use sha1::Sha1;

const API: &str = "https://api.tickets.yandex.net/api/agent";

#[derive(Debug)]
pub enum Day {
    Weekday,
    Weekend,
}

pub struct You {
    date: String,
    from: NaiveTime,
    to: NaiveTime,
    cookie: String,
    login: String,
    password: String,
    day: Day,
    total: bool,
    client: reqwest::Client,
    printer: Box<dyn Printer>,
}

impl You {
    pub fn new(
        date: &str,
        from: NaiveTime,
        to: NaiveTime,
        cookie: &str,
        login: &str,
        day: Day,
        password: &str,
        total: bool,
    ) -> Self {
        let client = reqwest::Client::new();
        let printer: Box<dyn Printer> = if total {
            Box::new(TotalPrinter)
        } else {
            Box::new(JustPrinter)
        };

        You {
            date: date.to_owned(),
            cookie: cookie.to_owned(),
            login: login.to_owned(),
            password: password.to_owned(),
            from,
            to,
            day,
            client,
            total,
            printer,
        }
    }

    pub async fn print_sessions(&self) {
        self.printer.as_ref().print(self).await;
    }

    pub fn identifier(&self) -> String {
        let mut password_hasher = Md5::new();
        password_hasher.update(&self.password);
        let password_hash = password_hasher.finalize();
        let password_hash = base16ct::lower::encode_string(&password_hash);

        let time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut passtime_hasher = Sha1::new();
        passtime_hasher.update(format!("{} {}", password_hash, time));
        let passtime_hash = base16ct::lower::encode_string(&passtime_hasher.finalize());

        format!("{}:{}:{}", self.login, passtime_hash, time)
    }

    pub async fn get_sessions(&self) -> HashMap<NaiveTime, String> {
        let url = format!("https://cms.tickets.yandex.net/reports/tickets/activity?report=1&from={0}&to={0}&activity_id=7552154&ext=0", self.date);
        let html = self
            .client
            .get(url)
            .header("Cookie", &self.cookie)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let dom = tl::parse(&html, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let hrefs: Vec<_> = dom.query_selector("a[href]").unwrap().collect();

        let mut sessions = HashMap::new();
        let mut time = match self.day {
            Day::Weekday => NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            Day::Weekend => NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        };
        let mut i = 33;
        while let Some(href) = hrefs.get(i) {
            let href = href.get(parser).unwrap().outer_html(parser);

            let session_id = href[33..40].to_owned();
            sessions.insert(time, session_id);
            time += Duration::minutes(30);

            i += 1;
        }

        sessions
    }

    async fn get_session_info(&self, event_id: &str) -> SessionInfo {
        let url = format!("https://cms.tickets.yandex.net/reports/ac/event?report=1&event_id={event_id}&dashboard=1&ext=0");
        let html = self
            .client
            .get(url)
            .header("Cookie", &self.cookie)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let dom = tl::parse(&html, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut element = dom.query_selector("td").unwrap();
        element.next().unwrap();

        let sold = element
            .next()
            .unwrap()
            .get(parser)
            .unwrap()
            .inner_text(parser)
            .parse::<u32>()
            .unwrap();

        let passed = element
            .next()
            .unwrap()
            .get(parser)
            .unwrap()
            .inner_text(parser)
            .parse::<u32>()
            .unwrap();

        let left = sold - passed;

        SessionInfo::new(sold, passed, left)
    }
}

#[derive(Default, Debug, Table)]
struct SessionInfo {
    #[table(title = "Время", justify = "Justify::Center")]
    time: String,
    #[table(title = "Продано", justify = "Justify::Center")]
    sold: u32,
    #[table(title = "Пришли", justify = "Justify::Center")]
    passed: u32,
    #[table(title = "Осталось", justify = "Justify::Center")]
    left: u32,
}

impl SessionInfo {
    fn new(sold: u32, passed: u32, left: u32) -> Self {
        SessionInfo {
            time: "".to_owned(),
            sold,
            passed,
            left,
        }
    }
}

impl Display for SessionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.time, self.sold, self.passed, self.left
        )
    }
}

impl<'a> AddAssign<&'a Self> for SessionInfo {
    fn add_assign(&mut self, rhs: &'a Self) {
        self.left += rhs.left;
        self.passed += rhs.passed;
        self.sold += rhs.sold;
    }
}

#[async_trait]
trait Printer: Sync + Send {
    async fn print(&self, you: &You);
}

struct TotalPrinter;
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

        print_stdout(sessions_info.with_title()).unwrap();
    }
}

struct JustPrinter;
#[async_trait]
impl Printer for JustPrinter {
    async fn print(&self, you: &You) {
        let sessions = you.get_sessions().await;

        let mut sessions_info = vec![];

        let mut time = you.from;

        while time <= you.to {
            let mut info = you.get_session_info(sessions.get(&time).unwrap()).await;
            info.time = time.format("%H:%M").to_string();

            sessions_info.push(info);

            time += Duration::minutes(30);
        }

        print_stdout(sessions_info.with_title()).unwrap();
    }
}
