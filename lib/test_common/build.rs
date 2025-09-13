// (C) COPYRIGHT 2018 TECHNOLUTION BV, GOUDA NL

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// This build script ensures the binaries of the test subjects (programs that are used as input for the tests)
/// are built for debug and release.
/// These subjects are used in tests that verify the tool works on a (new) particular Rust version.
/// These tests perform regression testing on the tool itself as well as the Rust compiler.
/// Changes in the Rust compiler that break the tool should be detected by tests
/// on these projects.
use std::path::Path;
use std::process::Command;

const RES_PATH: &str = "test_subjects";
const BUILD_MODE_ARGS: &[Option<&str>] = &[None, Some("--release")];

fn main() {
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    let grandparent_dir = current_dir
        .parent()
        .expect("Current directory has no parent")
        .parent()
        .expect("Current directory has no grandparent");

    let test_subjects_dir = Path::join(grandparent_dir, Path::new(RES_PATH));

    BUILD_MODE_ARGS.iter().for_each(|arg| {
        // clean the dir to force a fresh build
        let subjects_clean_status = Command::new("cargo")
            .current_dir(test_subjects_dir.clone())
            .arg("clean")
            .status()
            .expect("Cleaning test subject dir did not produce any output");

        if !subjects_clean_status.success() {
            panic!("Could not clean test subjects, manual intervention needed");
        }

        // rebuild the dir
        let mut cargo = Command::new("cargo");

        cargo.current_dir(test_subjects_dir.clone());

        cargo.arg("build");
        cargo.arg("--target");
        cargo.arg("x86_64-unknown-linux-gnu");

        if let Some(arg) = arg {
            cargo.arg(arg);
        }

        let subjects_build_status = cargo
            .status()
            .expect("Building of test subjects did not produce any output");

        if !subjects_build_status.success() {
            panic!("Could not build test subjects, manual intervention needed");
        }
    })
}