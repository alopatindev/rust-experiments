extern crate hyper;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use self::hyper::{Client, Url};
use self::hyper::client::Response;
use self::hyper::error;
use self::hyper::status::StatusCode;

const BUFFER_SIZE: usize = 4096;

pub fn download(url: &str,
                output_document: Option<String>,
                continue_partial: bool)
                -> Result<(), String> {
    match make_request(url) {
        Ok(mut response) => process_response(&mut response, output_document),
        Err(text) => Err(text.to_string()),
    }
}

fn make_request(url: &str) -> error::Result<Response> {
    let client = Client::new();
    client.get(url).send()
}

fn process_response(response: &mut Response,
                    output_document: Option<String>)
                    -> Result<(), String> {
    if response.status == StatusCode::Ok {
        let file_name = output_document.unwrap_or_else(|| response.url.to_file_name());
        match File::create(file_name) {
            Ok(mut file) => {
                let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
                loop {
                    let size = response.read(&mut buffer).unwrap();
                    if size == 0 {
                        break;
                    } else {
                        let _ = file.write(&buffer[0..size]).unwrap();
                    }
                }
                let _ = file.sync_all();
            }
            Err(_) => {}
        }
        Ok(())
    } else {
        Err(response.status.to_string())
    }
}

trait UrlFileName {
    fn to_file_name(&self) -> String;
}

impl UrlFileName for Url {
    fn to_file_name(&self) -> String {
        let path = self.path();
        let result = Path::new(path).file_name();
        if let Some(result) = result {
            if let Some(result) = result.to_str() {
                if !result.is_empty() {
                    return result.to_string();
                }
            }
        }

        "index.html".to_string()
    }
}
