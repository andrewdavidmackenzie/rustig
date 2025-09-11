#![allow(unexpected_cfgs)]

pub use error_chain::bail;
use error_chain::error_chain;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        CallGraph(::callgraph::errors::Error, ::callgraph::errors::ErrorKind);
    }

    errors {
        IOError(path: String) {
                    description("Binary file not found.")
                    display("File not found `{}`", path)
            }
    }
}