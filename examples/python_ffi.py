#!/usr/bin/env python3
"""
Python FFI Example for Recog
Demonstrates how to use the Recog library from Python via ctypes
"""

import ctypes
import json
from pathlib import Path

# Load the shared library
# On macOS: librecog.dylib, Linux: librecog.so, Windows: recog.dll
lib_path = Path(__file__).parent.parent / "target" / "release" / "librecog.dylib"
recog = ctypes.CDLL(str(lib_path))

# Define function signatures
recog.recog_new.argtypes = [ctypes.c_char_p]
recog.recog_new.restype = ctypes.c_void_p

recog.recog_match.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
recog.recog_match.restype = ctypes.c_char_p

recog.recog_free.argtypes = [ctypes.c_void_p]
recog.recog_free.restype = None

recog.recog_string_free.argtypes = [ctypes.c_char_p]
recog.recog_string_free.restype = None


class RecogMatcher:
    """Python wrapper for Recog FFI"""
    
    def __init__(self, xml_content: str):
        """Initialize matcher with XML fingerprint database"""
        self.ptr = recog.recog_new(xml_content.encode('utf-8'))
        if not self.ptr:
            raise RuntimeError("Failed to create matcher")
    
    def match(self, text: str) -> list:
        """Match text against fingerprints, returns list of matches"""
        result_ptr = recog.recog_match(self.ptr, text.encode('utf-8'))
        if not result_ptr:
            return []
        
        try:
            result_str = ctypes.string_at(result_ptr).decode('utf-8')
            return json.loads(result_str)
        finally:
            recog.recog_string_free(result_ptr)
    
    def __del__(self):
        """Clean up on deletion"""
        if hasattr(self, 'ptr') and self.ptr:
            recog.recog_free(self.ptr)


def main():
    # Sample XML fingerprint database
    xml = """
        <fingerprints>
            <fingerprint pattern="^Apache/([\\d.]+)" description="Apache HTTP Server">
                <param pos="1" name="version"/>
            </fingerprint>
            <fingerprint pattern="^nginx/([\\d.]+)" description="Nginx">
                <param pos="1" name="version"/>
            </fingerprint>
            <fingerprint pattern="^Microsoft-IIS/([\\d.]+)" description="Microsoft IIS">
                <param pos="1" name="version"/>
            </fingerprint>
        </fingerprints>
    """
    
    # Create matcher
    matcher = RecogMatcher(xml)
    
    # Test various banners
    test_banners = [
        "Server: Apache/2.4.41",
        "Server: nginx/1.18.0",
        "Server: Microsoft-IIS/10.0",
        "Server: Unknown/1.0"
    ]
    
    print("Recog Python FFI Demo")
    print("=" * 50)
    print()
    
    for banner in test_banners:
        results = matcher.match(banner)
        print(f"Banner: {banner}")
        
        if results:
            for match in results:
                print(f"  ✓ {match['description']}")
                print(f"    Parameters: {json.dumps(match['params'], indent=6)}")
        else:
            print("  ✗ No match")
        print()


if __name__ == "__main__":
    main()
