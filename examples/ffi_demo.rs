// Example: Using Recog FFI bindings from Rust
// In practice, this would be called from C/Python/Ruby/Node.js

use std::ffi::CString;

// Import FFI functions from the recog crate
use recog::ffi::{recog_free, recog_match, recog_new, recog_string_free};

fn main() {
    let xml = r#"
        <fingerprints>
            <fingerprint pattern="^Apache/([\d.]+)" description="Apache HTTP Server">
                <param pos="1" name="version"/>
            </fingerprint>
            <fingerprint pattern="^nginx/([\d.]+)" description="Nginx">
                <param pos="1" name="version"/>
            </fingerprint>
        </fingerprints>
    "#;

    unsafe {
        // Create matcher
        let xml_cstr = CString::new(xml).unwrap();
        let matcher = recog_new(xml_cstr.as_ptr());

        if matcher.is_null() {
            eprintln!("Failed to create matcher");
            return;
        }

        // Match some text
        let text = CString::new("Server: Apache/2.4.41").unwrap();
        let result_ptr = recog_match(matcher, text.as_ptr());

        if !result_ptr.is_null() {
            let result_cstr = std::ffi::CStr::from_ptr(result_ptr);
            if let Ok(result_str) = result_cstr.to_str() {
                println!("Match result: {}", result_str);
            }
            recog_string_free(result_ptr);
        }

        // Clean up
        recog_free(matcher);
    }

    println!("FFI example completed!");
}
