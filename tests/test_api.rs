// Copyright 2018-2019 the Deno authors. All rights reserved. MIT license.
#[macro_use]
extern crate lazy_static;

use rusty_v8 as v8;
use std::default::Default;
use std::sync::Mutex;

lazy_static! {
  static ref INIT_LOCK: Mutex<u32> = Mutex::new(0);
}

struct TestGuard {}

impl Drop for TestGuard {
  fn drop(&mut self) {
    // TODO shutdown process cleanly.
    /*
    *g -= 1;
    if *g  == 0 {
      unsafe { v8::V8::dispose() };
      v8::V8::shutdown_platform();
    }
    drop(g);
    */
  }
}

pub(crate) fn setup() -> TestGuard {
  let mut g = INIT_LOCK.lock().unwrap();
  *g += 1;
  if *g == 1 {
    v8::V8::initialize_platform(v8::platform::new_default_platform());
    v8::V8::initialize();
  }
  drop(g);
  TestGuard {}
}

#[test]
fn handle_scope_nested() {
  let g = setup();
  let mut params = v8::Isolate::create_params();
  params.set_array_buffer_allocator(
    v8::array_buffer::Allocator::new_default_allocator(),
  );
  let mut isolate = v8::Isolate::new(params);
  let mut locker = v8::Locker::new(&mut isolate);
  v8::HandleScope::enter(&mut locker, |scope| {
    v8::HandleScope::enter(scope, |_scope| {});
  });
  drop(g);
}

#[test]
#[allow(clippy::float_cmp)]
fn handle_scope_numbers() {
  let g = setup();
  let mut params = v8::Isolate::create_params();
  params.set_array_buffer_allocator(
    v8::array_buffer::Allocator::new_default_allocator(),
  );
  let mut isolate = v8::Isolate::new(params);
  let mut locker = v8::Locker::new(&mut isolate);
  v8::HandleScope::enter(&mut locker, |scope| {
    let l1 = v8::Integer::new(scope, -123);
    let l2 = v8::Integer::new_from_unsigned(scope, 456);
    v8::HandleScope::enter(scope, |scope2| {
      let l3 = v8::Number::new(scope2, 78.9);
      assert_eq!(l1.value(), -123);
      assert_eq!(l2.value(), 456);
      assert_eq!(l3.value(), 78.9);
      assert_eq!(v8::Number::value(&l1), -123f64);
      assert_eq!(v8::Number::value(&l2), 456f64);
    });
  });
  drop(g);
}

#[test]
fn test_string() {
  setup();
  let mut params = v8::Isolate::create_params();
  params.set_array_buffer_allocator(
    v8::array_buffer::Allocator::new_default_allocator(),
  );
  let mut isolate = v8::Isolate::new(params);
  let mut locker = v8::Locker::new(&mut isolate);
  v8::HandleScope::enter(&mut locker, |scope| {
    let reference = "Hello 🦕 world!";
    let local =
      v8::String::new(scope, reference, v8::NewStringType::Normal).unwrap();
    assert_eq!(reference, local.to_rust_string_lossy(scope));
  });
}

#[test]
fn script_compile_and_run() {
  setup();
  let mut params = v8::Isolate::create_params();
  params.set_array_buffer_allocator(
    v8::array_buffer::Allocator::new_default_allocator(),
  );
  let mut isolate = v8::Isolate::new(params);
  let mut locker = v8::Locker::new(&mut isolate);

  v8::HandleScope::enter(&mut locker, |s| {
    let mut context = v8::Context::new(s);
    context.enter();
    let source =
      v8::String::new(s, "'Hello ' + 13 + 'th planet'", Default::default())
        .unwrap();
    let mut script = v8::Script::compile(s, context, source, None).unwrap();
    source.to_rust_string_lossy(s);
    let result = script.run(s, context).unwrap();
    // TODO: safer casts.
    let result: v8::Local<v8::String> =
      unsafe { std::mem::transmute_copy(&result) };
    assert_eq!(result.to_rust_string_lossy(s), "Hello 13th planet");
  });
}

#[test]
fn get_version() {
  assert!(v8::V8::get_version().len() > 3);
}

#[test]
fn set_flags_from_command_line() {
  let r = v8::V8::set_flags_from_command_line(vec![
    "binaryname".to_string(),
    "--log-colour".to_string(),
    "--should-be-ignored".to_string(),
  ]);
  assert_eq!(
    r,
    vec!["binaryname".to_string(), "--should-be-ignored".to_string()]
  );
}

#[test]
fn inspector_string_view() {
  let chars = b"Hello world!";
  let view = v8::inspector::StringView::from(&chars[..]);

  assert_eq!(chars.len(), view.into_iter().len());
  assert_eq!(chars.len(), view.len());
  for (c1, c2) in chars.iter().copied().map(u16::from).zip(&view) {
    assert_eq!(c1, c2);
  }
}

#[test]
fn inspector_string_buffer() {
  let chars = b"Hello Venus!";
  let mut buf = {
    let src_view = v8::inspector::StringView::from(&chars[..]);
    v8::inspector::StringBuffer::create(&src_view)
  };
  let view = buf.as_mut().unwrap().string();

  assert_eq!(chars.len(), view.into_iter().len());
  assert_eq!(chars.len(), view.len());
  for (c1, c2) in chars.iter().copied().map(u16::from).zip(view) {
    assert_eq!(c1, c2);
  }
}
