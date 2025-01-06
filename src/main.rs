use clipboard_win::{formats, get_clipboard, set_clipboard};
use ctrlc;
use glob::glob;
use lazy_static::lazy_static;
use log::{error, warn};
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
    max_retries: u32,
    domains_to_unwrap: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check_interval: time::Duration::from_millis(1000),
            max_backoff: time::Duration::from_secs(30),
            max_retries: 3,
            domains_to_unwrap: vec![
                "safelinks.protection.outlook.com".to_string(),
                "nam.safelink.emails.azure.net".to_string(),
            ],
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

fn is_valid_content(content: &str) -> bool {
    !content.trim().is_empty() && content.len() < 1_000_000
}

fn extract_original_url(url: &Url) -> Option<String> {
    if url
        .host_str()?
        .ends_with("safelinks.protection.outlook.com")
    {
        url.query_pairs()
            .find(|(key, _)| key == "url")
            .map(|(_, value)| value.into_owned())
    } else if url.host_str()?.ends_with("safelink.emails.azure.net") {
        url.query_pairs()
            .find(|(key, _)| key == "destination")
            .map(|(_, value)| value.into_owned())
    } else {
        None
    }
}

fn replace<'a>(content: &'a str, config: &Config) -> Cow<'a, str> {
    lazy_static! {
        static ref MARKDOWN_LINK: Regex = Regex::new(r"(?x)\[(?P<linktext>.*?)\]\((?P<url>http.*?)\)").unwrap();
        static ref PLAIN_URL: Regex = Regex::new(r"(?x)(?P<url>https?://[^\s)]+)").unwrap();
    }
 
    let process_url = |url_str: &str, linktext: Option<&str>| -> String {
        if !is_valid_url(url_str) {
            return url_str.to_string();
        }
 
        match Url::parse(url_str) {
            Ok(parsed_url) => {
                if let Some(Host::Domain(host)) = parsed_url.host() {
                    if config.domains_to_unwrap.iter().any(|d| host.ends_with(d)) {
                        if let Some(url) = extract_original_url(&parsed_url) {
                            return match linktext {
                                Some(text) => format!("[{}]({})", text, url),
                                None => url
                            };
                        }
                    }
                }
                url_str.to_string()
            }
            Err(_) => url_str.to_string()
        }
    };
 
    let intermediate = MARKDOWN_LINK.replace_all(content, |c: &Captures| {
        let url = c.name("url").unwrap().as_str();
        let linktext = c.name("linktext").map(|m| m.as_str());
        process_url(url, linktext)
    }).into_owned();
 
    let final_result = PLAIN_URL.replace_all(&intermediate, |c: &Captures| {
        let url = c.name("url").unwrap().as_str();
        process_url(url, None)
    }).into_owned();
 
    Cow::Owned(final_result)
 }

fn run_clipboard_loop(config: &Config) {
    let mut backoff = config.check_interval;
    let mut last_content = String::new();
    let mut retry_count = 0;

    loop {
        match get_clipboard_content() {
            Ok(input) if is_valid_content(&input) && input != last_content => {
                retry_count = 0;
                let replaced = replace(&input, config);
                if let Err(e) = set_clipboard_content(&replaced) {
                    error!("Clipboard write error: {}", e);
                    backoff = (backoff * 2).min(config.max_backoff);
                } else {
                    last_content = input;
                    backoff = config.check_interval;
                }
            }
            Err(e) if !e.to_string().contains("1168") => {
                retry_count += 1;
                if retry_count > config.max_retries {
                    error!("Max retries exceeded, exiting");
                    std::process::exit(1);
                }
                warn!(
                    "Clipboard error (attempt {}/{}): {}",
                    retry_count, config.max_retries, e
                );
                backoff = (backoff * 2).min(config.max_backoff);
            }
            _ => {}
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

    ctrlc::set_handler(move || {
        println!("\nCleaning up...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    if let Ok(files) = recurse() {
        println!("Found {} markdown files", files.len());
    }

    run_clipboard_loop(&config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlook_url_extraction() {
        let url = Url::parse("https://outlook.safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com&data=...").unwrap();
        assert_eq!(
            extract_original_url(&url),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_azure_url_extraction() {
        let url = Url::parse("https://nam.safelink.emails.azure.net/redirect/?destination=https%3A%2F%2Fazuremsregistration.microsoft.com").unwrap();
        assert_eq!(
            extract_original_url(&url),
            Some("https://azuremsregistration.microsoft.com".to_string())
        );
    }
}
