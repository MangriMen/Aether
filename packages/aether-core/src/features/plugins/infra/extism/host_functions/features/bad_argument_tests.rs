//! A host function argument that does not decode is an answer, not a trap.

use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use aether_core_plugin_api::v0::{HostError, HostResult, HttpResponseDto, wire};
use async_trait::async_trait;
use extism::{Function, Manifest, Plugin, PluginBuilder, UserData, Wasm};

use super::super::{AllowedHostsSource, PluginContext};
use crate::{features::instance::PluginId, shared::limits::PluginSandboxLimits};

/// Exports `via_http_get` and `via_log`: each hands its input to that host function as it is,
/// then carries on and makes the answer its output.
const PLUGIN: &str = r#"(module
    (import "extism:host/env" "input_offset" (func $input_offset (result i64)))
    (import "extism:host/env" "length" (func $length (param i64) (result i64)))
    (import "extism:host/env" "output_set" (func $output_set (param i64 i64)))
    (import "extism:host/user" "http_get" (func $http_get (param i64) (result i64)))
    (import "extism:host/user" "log" (func $log (param i64) (result i64)))
    (func (export "via_http_get") (result i32)
        (local $out i64)
        (local.set $out (call $http_get (call $input_offset)))
        (call $output_set (local.get $out) (call $length (local.get $out)))
        (i32.const 0))
    (func (export "via_log") (result i32)
        (local $out i64)
        (local.set $out (call $log (call $input_offset)))
        (call $output_set (local.get $out) (call $length (local.get $out)))
        (i32.const 0)))"#;

/// A request that got past its argument would have to ask for the plugin's hosts.
struct NoHosts;

#[async_trait]
impl AllowedHostsSource for NoHosts {
    async fn current(
        &self,
        _plugin_id: &PluginId,
    ) -> crate::Result<crate::features::plugins::PluginHosts> {
        panic!("a request whose argument does not decode must not get this far")
    }
}

fn plugin() -> Plugin {
    let context = PluginContext {
        id: PluginId::from_static("test"),
        container: Weak::new(),
        hosts: Arc::new(NoHosts),
        limits: PluginSandboxLimits::default(),
        wasm_sha256: String::new(),
    };
    let function = |name: &str, handler: super::super::HostFn| {
        Function::new(
            name,
            [extism::PTR],
            [extism::PTR],
            UserData::new(context.clone()),
            handler,
        )
    };

    PluginBuilder::new(Manifest::new([Wasm::data(PLUGIN.as_bytes().to_vec())]))
        .with_functions([
            function("http_get", super::http_get),
            function("log", super::log),
        ])
        .build()
        .expect("the test plugin should load")
}

/// What the export answered, the call itself having to succeed.
fn answer<T: serde::de::DeserializeOwned>(export: &str, input: &[u8]) -> HostResult<T> {
    let output: Vec<u8> = plugin()
        .call(export, input)
        .expect("a bad argument is answered, the export is not aborted");

    wire::decode(&output).expect("the answer decodes")
}

fn invalid_argument_message<T: std::fmt::Debug>(answer: HostResult<T>) -> String {
    match answer {
        HostResult::Err(HostError::InvalidArgument { message }) => message,
        other => panic!("expected invalidArgument, got {other:?}"),
    }
}

/// Not msgpack at all: `0xc1` is the one byte msgpack never uses.
const GARBAGE: &[u8] = &[0xc1, 0xff, 0x00, 0x13];

#[test]
fn should_answer_invalid_argument_when_a_struct_argument_is_garbage() {
    let message = invalid_argument_message(answer::<HttpResponseDto>("via_http_get", GARBAGE));

    assert!(message.contains("`http_get`"), "{message}");
}

#[test]
fn should_answer_invalid_argument_when_a_struct_argument_misses_a_field() {
    // A map, but with the key renamed: `url` is missing.
    let renamed = wire::encode(&HashMap::from([("uri", "https://example.com/")])).unwrap();

    let message = invalid_argument_message(answer::<HttpResponseDto>("via_http_get", &renamed));

    assert!(message.contains("`http_get`"), "{message}");
    assert!(message.contains("url"), "{message}");
}

#[test]
fn should_answer_invalid_argument_when_no_argument_is_sent() {
    let message = invalid_argument_message(answer::<()>("via_log", &[]));

    assert!(message.contains("`log`"), "{message}");
}

#[test]
fn should_answer_invalid_argument_when_an_enum_value_is_unknown() {
    let unknown_level =
        wire::encode(&HashMap::from([("level", "loud"), ("message", "hello")])).unwrap();

    let message = invalid_argument_message(answer::<()>("via_log", &unknown_level));

    assert!(message.contains("`log`"), "{message}");
}
