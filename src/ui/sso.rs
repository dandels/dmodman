use crate::api::sso::*;
use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub async fn start_apikey_flow() -> Option<String> {
    println!("dmodman requires an API key to work.");
    println!("Would you like to create one?");
    println!("[y]es, [n]o");

    let mut yes = read_y_n();
    if !yes {
        return None;
    }

    let mut sso_client;
    loop {
        match SsoClient::new().await {
            Ok(c) => {
                sso_client = c;
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                println!("Failed to connect to Nexus.");
                println!("Would you like to retry?");
                println!("[y]es, [n]o");
                yes = read_y_n();
            }
        }
    }

    while yes {
        match sso_client.start_flow().await {
            Ok(()) => {
                println!("Succesfully connected to Nexus.");
                println!("Open the following URL in your browser to authorise dmodman.");
                println!("{}", sso_client.get_url());
                match sso_client.wait_apikey_response().await {
                    Ok(sso_resp) => {
                        if sso_resp.data.api_key.is_some() {
                            if !sso_resp.success {
                                println!("Nexus reported failure despite returning API key.");
                            }
                            return sso_resp.data.api_key;
                        } else if sso_resp.success {
                            println!("Nexus reported success despite returning no API key.");
                        } else {
                            println!("Nexus reported failure.");
                        }
                        if let Some(err_msg) = sso_resp.error {
                            println!("Error from Nexus: \"{}\"", err_msg);
                        }
                    }
                    Err(e) => {
                        println!("Failed to get API key.");
                        println!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                println!("Failed to connect to Nexus.");
            }
        }
        println!("Would you like to retry?");
        println!("[y]es, [n]o");
        yes = read_y_n();
    }
    let _ = sso_client.close_connection().await;
    None
}

fn read_y_n() -> bool {
    /* Read y/n without waiting for the user to press return.
     * Entering raw mode messes with stdout, so we can't println until it's dropped. */
    let stdout = std::io::stdout().into_raw_mode().unwrap();
    let stdin = std::io::stdin();
    let mut ret = false;

    for key in stdin.keys() {
        match key {
            Ok(Key::Char('y')) => {
                ret = true;
                break;
            }
            Ok(Key::Char('n')) | Ok(Key::Ctrl('c')) => {
                break;
            }
            Ok(_) => continue,
            Err(e) => {
                println!("{}", e);
                break;
            }
        };
    }
    stdout.lock().flush().unwrap();
    ret
}
