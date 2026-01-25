use crate::loader::load_fingerprints_from_xml;
use crate::Matcher;
use libc::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::ptr;

#[no_mangle]
pub unsafe extern "C" fn recog_new(xml_content: *const c_char) -> *mut c_void {
    if xml_content.is_null() {
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(xml_content) };
    let xml_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let db = match load_fingerprints_from_xml(xml_str) {
        Ok(db) => db,
        Err(_) => return ptr::null_mut(),
    };

    let matcher = Matcher::new(db);
    Box::into_raw(Box::new(matcher)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn recog_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr as *mut Matcher));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn recog_match(matcher_ptr: *mut c_void, text: *const c_char) -> *mut c_char {
    if matcher_ptr.is_null() || text.is_null() {
        return ptr::null_mut();
    }

    let matcher = unsafe { &*(matcher_ptr as *mut Matcher) };
    let c_str = unsafe { CStr::from_ptr(text) };
    let input_text = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let results = matcher.match_text(input_text);

    // Convert to JSON string
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let mut map = serde_json::Map::new();
            map.insert(
                "description".to_string(),
                serde_json::Value::String(r.fingerprint.description.clone()),
            );
            map.insert(
                "params".to_string(),
                serde_json::to_value(&r.params).unwrap_or_default(),
            );
            serde_json::Value::Object(map)
        })
        .collect();

    let json_str = serde_json::to_string(&json_results).unwrap_or_default();
    CString::new(json_str).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn recog_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}
