//! Single-owner OAuth service for the native desktop. No external interpreter is used.
pub mod config;
pub mod oauth;
mod pages;
mod server;
mod worker;
pub use server::run;
pub const SCOPE: &str = "vectorcraft:control";
