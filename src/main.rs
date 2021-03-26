#![windows_subsystem = "windows"]

use clipboard_win::{formats, get_clipboard, set_clipboard};
use std::collections::HashMap;
use std::{thread, time};
use url::{Host, Url};

fn main() {
    loop {
        let s: String = get_clipboard(formats::Unicode).expect("get_clipboard");

        match Url::parse(&s) {
            Ok(parsed_url) => match parsed_url.host() {
                Some(Host::Domain(host)) => {
                    if host.ends_with(".safelinks.protection.outlook.com") {
                        println!("Found {}", s);
                        let hash_query: HashMap<_, _> =
                            parsed_url.query_pairs().into_owned().collect();
                        match hash_query.get("url") {
                            Some(url) => {
                                set_clipboard(formats::Unicode, url).expect("set_clipboard");
                                println!("Set {}", url);
                            }
                            _ => {}
                        };
                    }
                }
                _ => {}
            },
            _ => {}
        }

        let sec = time::Duration::from_millis(1000);
        thread::sleep(sec);
    }
}
