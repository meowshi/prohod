use std::{collections::HashMap, env};

use chrono::{Duration, NaiveTime};
use md5::{Digest, Md5};
use sha1::Sha1;

use crate::{
    day::Day,
    printer::{just_printer::JustPrinter, total_printer::TotalPrinter, Printer},
    session_info::SessionInfo,
};

pub struct You {
    pub(crate) date: String,
    pub(crate) to: NaiveTime,
    pub(crate) from: NaiveTime,
    pub(crate) cookie: String,
    pub(crate) login: String,
    pub(crate) password: String,
    pub(crate) day: Day,
    pub(crate) client: reqwest::Client,
    pub(crate) printer: Box<dyn Printer>,
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
        let url = env::var("ACTIVITY_URL").unwrap().replace("{}", &self.date);
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

        let mut time = self.from;
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

    pub async fn get_session_info(&self, event_id: &str) -> SessionInfo {
        let url = env::var("EVENT_URL").unwrap().replace("{}", event_id);
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
