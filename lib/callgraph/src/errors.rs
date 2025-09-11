#![allow(unexpected_cfgs)]

pub use error_chain::bail;
use error_chain::error_chain;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors{
        NotSupported(functionality: String) {
                description("A binary was passed that requires unimplemented functionality.")
                display("Analysis aborted: binary contains {}, which is not supported. ", functionality)
        }
        ParseError(reason: String) {
                description("A file could not be parsed correctly.")
                display("Unable to parse file: {}", reason)
        }
        ReadError(path: String) {
                description("A file could not be read correctly.")
                display("Unable to read file `{}`", path)
        }
        IOError(path: String) {
                description("Binary file not found.")
                display("File not found `{}`", path)
        }
    }
}