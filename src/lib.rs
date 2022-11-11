#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]
#![allow(clippy::all)]

//! Rust wrapper for the `aws-c-sdkutils` library. For testing purposes only.
//! For interacting with AWS services, use the `aws-sdk-rust` crate instead.

use scratchstack_wrapper_aws_c_common::{aws_allocator, aws_byte_buf, aws_byte_cursor, aws_hash_table, aws_string};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests;