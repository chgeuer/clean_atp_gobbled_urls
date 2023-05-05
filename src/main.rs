#![windows_subsystem = "windows"]

use clipboard_win::{formats, get_clipboard, set_clipboard};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::collections::HashMap;
use std::{thread, time};
use url::{Host, Url};

// https://rust-lang-nursery.github.io/rust-cookbook/web/scraping.html#extract-all-unique-links-from-a-mediawiki-markup
// https://ledinhcuong99.medium.com/replace-string-using-regex-in-rust-4c64f9f38818
fn replace(content: &str) -> Cow<str> {
    lazy_static! {
        static ref MARKDOWN_URL: Regex = Regex::new(
            r"(?x)
            \[(?P<linktext>.*?)\]\((?P<url>http.*?)\)     # Some markdown link
            "
        )
        .unwrap();
    }

    MARKDOWN_URL.replace_all(content, |c: &Captures| {
        match (c.name("linktext"), c.name("url")) {
            (Some(linktext), Some(regex_match)) => match Url::parse(&regex_match.as_str()) {
                Ok(parsed_url) => match parsed_url.host() {
                    Some(Host::Domain(host)) => {
                        if host.ends_with(".safelinks.protection.outlook.com") {
                            let hash_query: HashMap<_, _> =
                                parsed_url.query_pairs().into_owned().collect();
                            match hash_query.get("url") {
                                Some(url) => {
                                    format!("[{}]({})", linktext.as_str(), url)
                                }
                                _ => format!("[{}]({})", linktext.as_str(), parsed_url.as_str()),
                            }
                        } else {
                            format!("[{}]({})", linktext.as_str(), regex_match.as_str())
                        }
                    }
                    _ => format!("[{}]({})", linktext.as_str(), regex_match.as_str()),
                },
                _ => format!("[{}]({})", linktext.as_str(), regex_match.as_str()),
            },
            _ => unreachable!(),
        }
    })
}

fn run_clipboard_loop() {
    loop {
        // grep --recursive --include '*.md' --files-with-matches nam06.safelinks.protection.outlook.com
        let input: String = get_clipboard(formats::Unicode).expect("get_clipboard");
        let replaced = replace(input.as_str());
        set_clipboard(formats::Unicode, replaced).expect("set_clipboard");

        let sec = time::Duration::from_millis(1000);
        thread::sleep(sec);
    }
}

fn main() {
    run_clipboard_loop()
}
