// (C) COPYRIGHT 2018 TECHNOLUTION BV, GOUDA NL

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Panic call output
//!
//! This module is concerned with formatting and outputting panic traces from [panic_analysis](../panic_analysis/fn.find_panics.html) to the standard output.
//!
//! ## Output modes
//! The library offers three different output modes:
//!
//! ### 1. Simple
//! The simple output mode prints the most relevant information about the panic trace. The output consists of one line per _panic_ trace.
//! In that trace, the first two functions of the trace are displayed (inlined functions ignored).
//! The given file name and line number should allow to find the place the _panic_ trace occurs in the code.
//!
//! The following example could be a line printed in simple mode:
//! ```text
//! <core::result::Result<T, E> as cli::errors::ResultExt<T>>::chain_err calls core::panicking::panic (stdlib@1.26.2) at /home/pc/rustig/<impl_error_chain_processed macros>:141
//! ```
//!
//! ### 2. Verbose.
//! The verbose output mode prints a full trace from the last function in the analysis target to the _panic_ handler, including inlined functions.
//! Furthermore, the cause for the _panic_ is given, if it could be determined (See [PanicPattern](../panic_analysis/enum.PanicPattern.html)). Also, a message is printed if the trace contains dynamic invocations, since then it could be a false positive.
//!
//! The following example could be a line printed in verbose mode:
//! ```text
//! --#001 --Pattern: Unwrap
//!
//!  0: <core::result::Result<T, E> as cli::errors::ResultExt<T>>::chain_err (rustig)
//!          at /home/pc/rustig/<impl_error_chain_processed macros>:141
//!          <inline <core::result::Result<T, E>>::map_err at /checkout/src/libcore/result.rs:500 >
//!          <inline <core::result::Result<T, E> as cli::errors::ResultExt<T>>::chain_err::{{closure}} at /home/pc/rustig/<impl_error_chain_processed macros>:144 >
//!          <inline cli::config_file::parse_config::{{closure}} at /home/pc/rustig/bin/cli/src/config_file.rs:60 >
//!          <inline <core::option::Option<T>>::unwrap at /checkout/src/libcore/macros.rs:20 >
//!  1: core::panicking::panic (stdlib@1.26.2)
//!          at /checkout/src/libcore/panicking.rs:51
//!  2: core::panicking::panic_fmt (stdlib@1.26.2)
//!          at /checkout/src/libcore/panicking.rs:72
//!  3: rust_begin_unwind (stdlib@1.26.2)
//!          at /checkout/src/libstd/panicking.rs:325
//!  4: std::panicking::begin_panic_fmt (stdlib@1.26.2)
//!
//! ### 3. JSON.
//! The same amount of information as verbose, but formatted as JSON.
//! ```

use std::cell::RefCell;
use panic_analysis::{PanicCallsCollection, PanicPattern};
use serde_json as json;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;
use serde_json::json;

/// Set of options on how to format the output.
pub struct OutputOptions {
    /// The silent flag for when no output should be printed.
    pub silent: bool,
    /// The verbose flag for command line output.
    pub verbose: bool,
    /// The JSON flag for command line output.
    pub json: bool,
}

/// A struct consisting of a vector containing the output streams
pub struct OutputStreamsCollection {
    pub streams: Vec<Box<dyn OutputStream>>,
}

/// Trait marking objects that are able to output the found panic paths in a particular format, to a particular destination
pub trait OutputStream {
    fn print_output(&self, panic_calls: &PanicCallsCollection);
}

/// Struct that handles simple console output formatting
#[derive(Debug, Clone)]
struct SimpleConsoleOutputStream {}

impl OutputStream for SimpleConsoleOutputStream {
    fn print_output(&self, panic_calls: &PanicCallsCollection) {
        for trace in &panic_calls.calls {
            println!("{}", trace)
        }
    }
}

/// Struct that handles verbose console output formatting
#[derive(Debug, Clone)]
struct VerboseConsoleOutputStream {}

impl OutputStream for VerboseConsoleOutputStream {
    fn print_output(&self, panic_calls: &PanicCallsCollection) {
        println!(
            "{} calls found that lead to panic!",
            &panic_calls.calls.len()
        );
        for (i, trace) in panic_calls.calls.iter().enumerate() {
            println!(
                "--#{:0width$} {:#}",
                i + 1,
                trace,
                width = (panic_calls.calls.len() as f64).log10().ceil() as usize,
            )
        }
    }
}

/// Struct that handles JSON console output formatting
#[derive(Debug, Clone)]
struct JsonConsoleOutputStream {}

impl OutputStream for JsonConsoleOutputStream {
    fn print_output(&self, panic_calls: &PanicCallsCollection) {
        let stream = io::stdout();
        for (i, trace) in panic_calls.calls.iter().enumerate() {
            let json = json!({
                "index" : i,
                "pattern" : match trace.pattern.borrow().deref() {
                    PanicPattern::Unrecognized => "unrecognized",
                    PanicPattern::DirectCall => "direct_call",
                    PanicPattern::Unwrap => "unwrap",
                    PanicPattern::Indexing => "indexing",
                    PanicPattern::Arithmetic => "arithmetic",
                },
                "message" : if let Some(message) = &trace.message { message.clone().into() } else { json::Value::Null },
                "dynamic_invocation" : trace.contains_dynamic_invocation,
                "backtrace" : json::Value::Array(
                    trace.backtrace.iter().enumerate().map(|(i, backtrace)| {
                            let procedure = backtrace.procedure.deref().borrow();
                            let invocation = backtrace.outgoing_invocation.as_ref().map(Rc::deref).map(RefCell::borrow);
                            json!({
                                "index" : i,
                                "procedure" : json!({
                                    "name" : procedure.name.clone(),
                                    "linkage_name" : procedure.linkage_name.clone(),
                                    "linkage_name_demangled" : procedure.linkage_name_demangled.clone(),
                                    "crate" : json!({
                                        "name" : procedure.defining_crate.name.clone(),
                                        "version" : if let Some(version) = &procedure.defining_crate.version { version.clone().into() } else { json::Value::Null },
                                    }),
                                    "location" : if let Some(location) = &procedure.location {
                                            json!({
                                                "file" : location.file.clone(),
                                                "line" : location.line,
                                            })
                                        } else {
                                            json::Value::Null
                                        },
                                    "is_entry" : procedure.attributes.entry_point,
                                    "is_reachable" : procedure.attributes.reachable_from_entry_point,
                                    "is_panic" : procedure.attributes.is_panic,
                                    "is_panic_origin" : procedure.attributes.is_panic_origin,
                                    "is_whitelisted" : procedure.attributes.whitelisted,
                                }),
                                "invocation" :
                                    if let Some(invocation) = invocation {
                                        json!({
                                            "type" : match invocation.invocation_type {
                                                callgraph::InvocationType::Direct => "direct",
                                                callgraph::InvocationType::ProcedureReference => "procedure",
                                                callgraph::InvocationType::VTable => "vtable",
                                                callgraph::InvocationType::Jump => "jump",
                                            },
                                            "is_whitelisted" : invocation.attributes.whitelisted,
                                            "frames" : json::Value::Array(
                                                invocation.frames.iter().enumerate().map(|(i, frame)| {
                                                    json!({
                                                        "index" : i,
                                                        "function" : frame.function_name.clone(),
                                                        "location" : json!({
                                                            "file" : frame.location.file.clone(),
                                                            "line" : frame.location.line,
                                                        }),
                                                        "crate" : json!({
                                                            "name" : frame.defining_crate.name.clone(),
                                                            "version" : if let Some(version) = &frame.defining_crate.version { version.clone().into() } else { json::Value::Null },
                                                        }),
                                                    })
                                                }).collect()),
                                        })
                                    } else {
                                        json::Value::Null
                                    },
                            })
                        }).collect()
                ),
            });
            {
                let mut stream = stream.lock();
                write!(&mut stream, "\n\n").unwrap();
                json::to_writer_pretty (&mut stream, &json).unwrap();
                write!(&mut stream, "\n\n").unwrap();
            }
        }
    }
}

fn get_output_streams(options: &OutputOptions) -> Box<OutputStreamsCollection> {
    let mut output_stream_vec: Vec<Box<dyn OutputStream>> = Vec::new();

    if options.silent {
        return Box::new(OutputStreamsCollection { streams: vec![] });
    }

    if options.json {
        output_stream_vec.push(Box::new(JsonConsoleOutputStream {}));
    } else if options.verbose {
        output_stream_vec.push(Box::new(VerboseConsoleOutputStream {}));
    } else {
        output_stream_vec.push(Box::new(SimpleConsoleOutputStream {}));
    }

    Box::new(OutputStreamsCollection {
        streams: output_stream_vec,
    })
}

/// Print the results to the standard output, in the format specified by the options parameter.
pub fn print_results(options: &OutputOptions, results: &PanicCallsCollection) {
    let output_streams = get_output_streams(&options);

    // Output results
    for outputstream in output_streams.streams {
        outputstream.print_output(&results)
    }
}
