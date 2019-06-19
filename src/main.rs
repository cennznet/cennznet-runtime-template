//! CENNZnet Runtime Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
mod cli;
mod service;

pub use substrate_cli::{error, IntoExit, VersionInfo};

fn run() -> cli::error::Result<()> {
    let version = VersionInfo {
        name: "CENNZnet Runtime Template Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "cennznet-runtime-template",
        author: "Centrality Developers",
        description: "CENNZnet Runtime Template",
        support_url: "https://github.com/cennznet/cennznet-runtime-template/issues",
    };
    cli::run(::std::env::args(), cli::Exit, version)
}

error_chain::quick_main!(run);
