# Recog Examples

This directory contains example code demonstrating how to use Recog in various environments.

## Examples Overview

### 1. **Analytics Demo** (`analytics_demo.rs`)
Demonstrates the fingerprint analytics tool for detecting overlapping patterns.

```bash
cargo run --example analytics_demo
```

This example shows how to:
- Load fingerprint databases
- Detect overlapping patterns that share common literals
- Identify potential redundancies in fingerprint definitions

### 2. **FFI Demo** (`ffi_demo.rs`)
Shows how to use the C FFI bindings from Rust (demonstrating the API structure).

```bash
cargo run --example ffi_demo
```

Features:
- Creating a matcher from XML
- Matching text via FFI
- Proper memory management

### 3. **Python FFI** (`python_ffi.py`)
Python bindings using ctypes to call the Recog C FFI.

First, build the shared library:
```bash
cargo build --release
```

Then run the Python example:
```bash
python3 examples/python_ffi.py
```

This demonstrates:
- Loading Recog from Python
- Wrapper class for ergonomic usage
- JSON result parsing

### 4. **WASM Demo** (`wasm_demo.html`)
Interactive web application using WebAssembly.

Build the WASM module:
```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build for web
wasm-pack build --target web --out-dir examples/pkg
```

Then serve the HTML file:
```bash
# Using Python's built-in server
python3 -m http.server 8000

# Or using any other static server
# Navigate to http://localhost:8000/examples/wasm_demo.html
```

Features:
- Browser-based fingerprint matching
- No server required
- Beautiful, responsive UI
- Real-time matching

## Building for Different Targets

### Standard Library
```bash
cargo build --release
```

### WebAssembly
```bash
wasm-pack build --target web
```

### Shared Library for FFI
```bash
cargo build --release --lib
# Output: target/release/librecog.{so|dylib|dll}
```

## Integration Patterns

### Node.js (via FFI)
```javascript
const ffi = require('ffi-napi');
const path = require('path');

const recog = ffi.Library(path.join(__dirname, '../target/release/librecog'), {
  'recog_new': ['pointer', ['string']],
  'recog_match': ['string', ['pointer', 'string']],
  'recog_free': ['void', ['pointer']],
  'recog_string_free': ['void', ['string']]
});

// Usage similar to Python example
```

### Ruby (via FFI)
```ruby
require 'ffi'

module Recog
  extend FFI::Library
  ffi_lib File.join(__dir__, '../target/release/librecog.dylib')
  
  attach_function :recog_new, [:string], :pointer
  attach_function :recog_match, [:pointer, :string], :string
  attach_function :recog_free, [:pointer], :void
  attach_function :recog_string_free, [:string], :void
end

# Usage similar to Python example
```

## Performance Notes

- **WASM**: Excellent performance in modern browsers, ~90% of native speed
- **FFI**: Near-native performance with minimal overhead
- **Parallel Matching**: Enabled by default via Rayon
- **Aho-Corasick**: Pre-filtering reduces regex evaluations significantly

## Additional Resources

- [WebAssembly Guide](https://rustwasm.github.io/docs/book/)
- [FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
