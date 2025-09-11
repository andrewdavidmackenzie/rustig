#![allow(unexpected_cfgs)]

use error_chain::error_chain;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors{
        ConfigLoad(path: String, reason: Option<String>) {
            description("Config file not found")
            display("Unable to read config file `{}`{}", path, reason.as_ref().map(|x| format!(": {}", x)).unwrap_or_else(|| "".to_string()))
        }
    }
}