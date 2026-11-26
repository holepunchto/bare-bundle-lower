#include <assert.h>
#include <bare.h>
#include <js.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// Implemented in Rust, see `src/lib.rs`.

typedef struct {
  const uint8_t *data;
  size_t len;
} bare_bundle_lower_string_t;

typedef struct {
  bare_bundle_lower_string_t target;
  bare_bundle_lower_string_t format;
} bare_bundle_lower_options_t;

typedef struct {
  bare_bundle_lower_string_t url;
  uint32_t start;
  uint32_t end;
  bare_bundle_lower_string_t message;
} bare_bundle_lower_diagnostic_t;

typedef struct {
  const uint8_t *data;
  size_t len;
  const bare_bundle_lower_diagnostic_t *diagnostics;
  size_t diagnostics_len;
  void *internal;
} bare_bundle_lower_result_t;

int
bare_bundle_lower(const bare_bundle_lower_options_t *options, const uint8_t *input, size_t len, bare_bundle_lower_result_t *result);

void
bare_bundle_lower_result_free(bare_bundle_lower_result_t *result);

static int
bare_bundle_lower__get_int64(js_env_t *env, js_value_t *value, int64_t *result) {
  int err;

  bool is_number;
  err = js_is_number(env, value, &is_number);
  assert(err == 0);

  if (!is_number) return -1;

  err = js_get_value_int64(env, value, result);
  assert(err == 0);

  return 0;
}

static int
bare_bundle_lower__get_string(js_env_t *env, js_value_t *value, bare_bundle_lower_string_t *result) {
  int err;

  bool is_string;
  err = js_is_string(env, value, &is_string);
  assert(err == 0);

  if (!is_string) return -1;

  size_t len;
  err = js_get_value_string_utf8(env, value, NULL, 0, &len);
  assert(err == 0);

  utf8_t *data = malloc(len + 1);

  if (data == NULL) return -1;

  err = js_get_value_string_utf8(env, value, data, len + 1, NULL);
  assert(err == 0);

  result->data = data;
  result->len = len;

  return 0;
}

static js_value_t *
bare_bundle_lower__create_diagnostics(js_env_t *env, const bare_bundle_lower_result_t *result) {
  int err;

  js_value_t *diagnostics;
  err = js_create_array_with_length(env, result->diagnostics_len, &diagnostics);
  assert(err == 0);

  for (size_t i = 0; i < result->diagnostics_len; i++) {
    const bare_bundle_lower_diagnostic_t *diagnostic = &result->diagnostics[i];

    js_value_t *entry;
    err = js_create_object(env, &entry);
    assert(err == 0);

    err = js_set_element(env, diagnostics, (uint32_t) i, entry);
    if (err < 0) return NULL;

    js_value_t *val;

    err = js_create_string_utf8(env, diagnostic->url.data, diagnostic->url.len, &val);
    if (err < 0) return NULL;

    err = js_set_named_property(env, entry, "url", val);
    if (err < 0) return NULL;

    err = js_create_uint32(env, diagnostic->start, &val);
    assert(err == 0);

    err = js_set_named_property(env, entry, "start", val);
    if (err < 0) return NULL;

    err = js_create_uint32(env, diagnostic->end, &val);
    assert(err == 0);

    err = js_set_named_property(env, entry, "end", val);
    if (err < 0) return NULL;

    err = js_create_string_utf8(env, diagnostic->message.data, diagnostic->message.len, &val);
    if (err < 0) return NULL;

    err = js_set_named_property(env, entry, "message", val);
    if (err < 0) return NULL;
  }

  return diagnostics;
}

static void
bare_bundle_lower__on_finalize(js_env_t *env, void *data, void *finalize_hint) {
  bare_bundle_lower_result_t *result = finalize_hint;

  bare_bundle_lower_result_free(result);

  free(result);
}

static js_value_t *
bare_bundle_lower_lower(js_env_t *env, js_callback_info_t *info) {
  int err;

  size_t argc = 5;
  js_value_t *argv[5];

  err = js_get_callback_info(env, info, &argc, argv, NULL, NULL);
  assert(err == 0);

  bool is_arraybuffer;
  err = js_is_arraybuffer(env, argv[0], &is_arraybuffer);
  assert(err == 0);

  bool is_sharedarraybuffer;
  err = js_is_sharedarraybuffer(env, argv[0], &is_sharedarraybuffer);
  assert(err == 0);

  if (!is_arraybuffer && !is_sharedarraybuffer) {
    err = js_throw_type_error(env, NULL, "Input must be a buffer");
    assert(err == 0);

    return NULL;
  }

  void *data;
  size_t byte_len;

  if (is_sharedarraybuffer) {
    err = js_get_sharedarraybuffer_info(env, argv[0], &data, &byte_len);
  } else {
    err = js_get_arraybuffer_info(env, argv[0], &data, &byte_len);
  }

  assert(err == 0);

  int64_t offset;
  if (bare_bundle_lower__get_int64(env, argv[1], &offset) < 0) {
    err = js_throw_type_error(env, NULL, "Offset must be a number");
    assert(err == 0);

    return NULL;
  }

  int64_t len;
  if (bare_bundle_lower__get_int64(env, argv[2], &len) < 0) {
    err = js_throw_type_error(env, NULL, "Length must be a number");
    assert(err == 0);

    return NULL;
  }

  if (offset < 0 || len < 0 || (uint64_t) offset + (uint64_t) len > byte_len) {
    err = js_throw_range_error(env, NULL, "Offset and length are out of bounds");
    assert(err == 0);

    return NULL;
  }

  bare_bundle_lower_string_t target;
  if (bare_bundle_lower__get_string(env, argv[3], &target) < 0) {
    err = js_throw_type_error(env, NULL, "Target must be a string");
    assert(err == 0);

    return NULL;
  }

  bare_bundle_lower_string_t format;
  if (bare_bundle_lower__get_string(env, argv[4], &format) < 0) {
    free((void *) target.data);

    err = js_throw_type_error(env, NULL, "Format must be a string");
    assert(err == 0);

    return NULL;
  }

  const uint8_t *input = (const uint8_t *) data + offset;

  // The lowering reads the input as UTF-8 text and must not see it change
  // while it works, which shared memory would allow. Lower a private copy.
  uint8_t *copy = NULL;

  if (is_sharedarraybuffer && len > 0) {
    copy = malloc((size_t) len);

    if (copy == NULL) {
      free((void *) target.data);
      free((void *) format.data);

      err = js_throw_error(env, NULL, "Out of memory");
      assert(err == 0);

      return NULL;
    }

    memcpy(copy, input, (size_t) len);

    input = copy;
  }

  bare_bundle_lower_options_t options = {
    .target = target,
    .format = format,
  };

  bare_bundle_lower_result_t result;
  int status = bare_bundle_lower(&options, input, (size_t) len, &result);

  free(copy);

  free((void *) target.data);
  free((void *) format.data);

  js_value_t *output;
  err = js_create_object(env, &output);
  assert(err == 0);

  if (status == 0) {
    bare_bundle_lower_result_t *owned = malloc(sizeof(bare_bundle_lower_result_t));

    if (owned == NULL) {
      bare_bundle_lower_result_free(&result);

      err = js_throw_error(env, NULL, "Out of memory");
      assert(err == 0);

      return NULL;
    }

    *owned = result;

    js_value_t *arraybuffer;
    err = js_create_external_arraybuffer(env, (void *) owned->data, owned->len, bare_bundle_lower__on_finalize, owned, &arraybuffer);

    if (err < 0) {
      bare_bundle_lower__on_finalize(env, NULL, owned);

      return NULL;
    }

    err = js_set_named_property(env, output, "buffer", arraybuffer);
    if (err < 0) return NULL;
  } else {
    js_value_t *diagnostics = bare_bundle_lower__create_diagnostics(env, &result);

    bare_bundle_lower_result_free(&result);

    if (diagnostics == NULL) return NULL;

    err = js_set_named_property(env, output, "diagnostics", diagnostics);
    if (err < 0) return NULL;
  }

  return output;
}

static js_value_t *
bare_bundle_lower_exports(js_env_t *env, js_value_t *exports) {
  int err;

#define V(name, fn) \
  { \
    js_value_t *val; \
    err = js_create_function(env, name, -1, fn, NULL, &val); \
    assert(err == 0); \
    err = js_set_named_property(env, exports, name, val); \
    assert(err == 0); \
  }

  V("lower", bare_bundle_lower_lower)
#undef V

  return exports;
}

BARE_MODULE(bare_bundle_lower, bare_bundle_lower_exports)
