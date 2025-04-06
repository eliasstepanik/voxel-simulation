use spacetimedb_sdk::{credentials, Error, Identity};
use crate::module_bindings::{DbConnection, ErrorContext};


pub fn creds_store() -> credentials::File {
    credentials::File::new("token")
}

/// Our `on_connect` callback: save our credentials to a file.
pub fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
    if let Err(e) = creds_store().save(token) {
        eprintln!("Failed to save credentials: {:?}", e);
    }
    
}

/// Our `on_connect_error` callback: print the error, then exit the process.
pub fn on_connect_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Connection error: {:?}", err);
    std::process::exit(1);
}

/// Our `on_disconnect` callback: print a note, then exit the process.
pub fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
    if let Some(err) = err {
        eprintln!("Disconnected: {}", err);
        std::process::exit(1);
    } else {
        println!("Disconnected.");
        std::process::exit(0);
    }
}
