use clipboard_win::{formats, get_clipboard, set_clipboard};
use glob::glob;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::io;
use std::{thread, time};
use url::{Host, Url};

type ClipboardResult<T> = Result<T, io::Error>;

#[derive(Debug)]
struct Config {
    check_interval: time::Duration,
    max_backoff: time::Duration,
    domains_to_unwrap: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check_interval: time::Duration::from_millis(1000),
            max_backoff: time::Duration::from_secs(30),
            domains_to_unwrap: vec!["safelinks.protection.outlook.com".to_string()],
        }
    }
}

fn get_clipboard_content() -> ClipboardResult<String> {
    get_clipboard(formats::Unicode).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

fn set_clipboard_content(content: &str) -> ClipboardResult<()> {
    set_clipboard(formats::Unicode, content)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

fn extract_original_url(url: &Url) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == "url")
        .map(|(_, value)| value.into_owned())
}

fn replace<'a>(content: &'a str, config: &Config) -> Cow<'a, str> {
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
            (Some(linktext), Some(regex_match)) => {
                let url_str = regex_match.as_str();
                if !is_valid_url(url_str) {
                    return format!("[{}]({})", linktext.as_str(), url_str);
                }

                match Url::parse(url_str) {
                    Ok(parsed_url) => match parsed_url.host() {
                        Some(Host::Domain(host)) => {
                            if config.domains_to_unwrap.iter().any(|d| host.ends_with(d)) {
                                match extract_original_url(&parsed_url) {
                                    Some(url) => format!("[{}]({})", linktext.as_str(), url),
                                    None => format!("[{}]({})", linktext.as_str(), url_str),
                                }
                            } else {
                                format!("[{}]({})", linktext.as_str(), url_str)
                            }
                        }
                        _ => format!("[{}]({})", linktext.as_str(), url_str),
                    },
                    Err(_) => format!("[{}]({})", linktext.as_str(), url_str),
                }
            }
            _ => unreachable!(),
        }
    })
}

fn run_clipboard_loop(config: &Config) {
    let mut backoff = config.check_interval;
    let mut last_content = String::new();

    loop {
        match get_clipboard_content() {
            Ok(input) => {
                if input != last_content {
                    let replaced = replace(&input, config);
                    if let Err(e) = set_clipboard_content(&replaced) {
                        eprintln!("Clipboard write error: {:?}", e);
                        backoff = (backoff * 2).min(config.max_backoff);
                    } else {
                        last_content = input;
                        backoff = config.check_interval;
                    }
                }
            }
            Err(e) => {
                if !e.to_string().contains("1168") {
                    // Skip empty clipboard errors
                    eprintln!("Clipboard read error: {:?}", e);
                    backoff = (backoff * 2).min(config.max_backoff);
                }
            }
        }
        thread::sleep(backoff);
    }
}

fn recurse() -> Result<Vec<String>, glob::PatternError> {
    let mut files = Vec::new();
    for entry in glob("**/*.md")? {
        match entry {
            Ok(path) => {
                println!("Found: {:?}", path.display());
                files.push(path.to_string_lossy().into_owned());
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
    Ok(files)
}

fn main() {
    let config = Config::default();

    if let Ok(files) = recurse() {
        println!("Found {} markdown files", files.len());
    }

    run_clipboard_loop(&config);
}
