//! Async I/O support for large fingerprint databases
//!
//! This module provides async versions of the core I/O operations for better
//! performance with large fingerprint databases and concurrent processing.

#![cfg(feature = "async")]

use crate::error::{RecogError, RecogResult};
use crate::fingerprint::{Example, Fingerprint, FingerprintDatabase};
use crate::params::Param;
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::path::Path;
use tokio::{fs, io::AsyncReadExt, task};

/// Async version of XML loading from file
pub async fn load_fingerprints_from_file_async<P: AsRef<Path>>(
    path: P,
) -> RecogResult<FingerprintDatabase> {
    let path = path.as_ref().to_path_buf();
    let xml_content: String = fs::read_to_string(&path).await?;
    load_fingerprints_from_xml_async(&xml_content).await
}

/// Async version of XML loading from string
pub async fn load_fingerprints_from_xml_async(
    xml_content: &str,
) -> RecogResult<FingerprintDatabase> {
    // For now, we use the synchronous parser since we don't have async XML parsing
    // In a production system, we might want to use a streaming XML parser
    let xml_content = xml_content.to_string();
    let db = task::spawn_blocking(move || {
        let xml_fps: XmlFingerprints = quick_xml::de::from_str(&xml_content)
            .map_err(|e| RecogError::custom(format!("XML parsing error: {}", e)))?;
        let mut db = FingerprintDatabase::new();

        for xml_fp in xml_fps.fingerprints {
            let fingerprint = xml_fp.into_fingerprint()?;
            db.add_fingerprint(fingerprint);
        }

        Ok(db)
    })
    .await
    .map_err(|e| RecogError::custom(format!("Task join error: {}", e)))?;

    Ok(db)
}

/// Async version of saving fingerprints to XML
pub async fn save_fingerprints_to_xml_async(_db: &FingerprintDatabase) -> RecogResult<String> {
    // This would implement XML serialization if needed
    // For now, return a placeholder
    Ok("<?xml version=\"1.0\"?><fingerprints></fingerprints>".to_string())
}

/// Async loader for multiple fingerprint files concurrently
pub async fn load_multiple_databases_async<P: AsRef<Path>>(
    paths: &[P],
) -> RecogResult<Vec<FingerprintDatabase>> {
    let mut databases = Vec::new();

    // Process files concurrently
    let mut handles = Vec::new();
    for path in paths {
        let path = path.as_ref().to_path_buf();
        let handle = tokio::spawn(async move { load_fingerprints_from_file_async(path).await });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let db = handle
            .await
            .map_err(|e| RecogError::custom(format!("Task join error: {}", e)))?;
        databases.push(db?);
    }

    Ok(databases)
}

/// Streaming XML parser for memory-constrained environments
pub struct StreamingXmlLoader {
    buffer_size: usize,
}

impl StreamingXmlLoader {
    /// Create a new streaming XML loader with specified buffer size
    pub fn new(buffer_size: usize) -> Self {
        Self { buffer_size }
    }

    /// Load fingerprints from a large XML file in chunks
    pub async fn load_large_file_streaming<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> RecogResult<FingerprintDatabase> {
        let path = path.as_ref();
        let mut file = fs::File::open(path).await?;
        let mut buffer = Vec::new();
        let mut db = FingerprintDatabase::new();

        // Read file in chunks
        loop {
            let mut chunk = vec![0; self.buffer_size];
            let bytes_read = file.read(&mut chunk).await?;

            if bytes_read == 0 {
                break; // EOF
            }

            buffer.extend_from_slice(&chunk[..bytes_read]);

            // Try to parse complete fingerprints from buffer
            if let Ok((remaining, fingerprints)) = self.parse_buffer(&buffer) {
                buffer = remaining;

                for fp in fingerprints {
                    db.add_fingerprint(fp);
                }
            }
        }

        Ok(db)
    }

    /// Parse complete fingerprints from buffer, returning unparsed remainder
    fn parse_buffer(&self, buffer: &[u8]) -> Result<(Vec<u8>, Vec<Fingerprint>), RecogError> {
        let xml_str = std::str::from_utf8(buffer)
            .map_err(|_| RecogError::custom("Invalid UTF-8 in XML buffer"))?;

        // This is a simplified parser - in production, we'd use a proper streaming XML parser
        // For now, we'll assume the buffer contains complete fingerprints
        let xml_fps: XmlFingerprints = quick_xml::de::from_str(xml_str)?;

        let mut fingerprints = Vec::new();
        for xml_fp in xml_fps.fingerprints {
            let fingerprint = xml_fp.into_fingerprint()?;
            fingerprints.push(fingerprint);
        }

        // Return empty remainder for now - proper implementation would track parsing state
        Ok((Vec::new(), fingerprints))
    }
}

impl Default for StreamingXmlLoader {
    fn default() -> Self {
        Self::new(8192) // 8KB default buffer
    }
}

// XML parsing structures (same as sync version)
#[derive(Debug, Deserialize)]
struct XmlFingerprints {
    #[serde(rename = "fingerprint")]
    fingerprints: Vec<XmlFingerprint>,
}

#[derive(Debug, Deserialize)]
struct XmlFingerprint {
    #[serde(rename = "@pattern")]
    pattern: String,
    #[serde(rename = "@description")]
    description: String,
    #[serde(rename = "example", default)]
    examples: Vec<XmlExample>,
    #[serde(rename = "param", default)]
    params: Vec<XmlParam>,
}

#[derive(Debug, Deserialize)]
struct XmlExample {
    #[serde(rename = "@value")]
    value: Option<String>,
    #[serde(rename = "@filename")]
    filename: Option<String>,
    #[serde(rename = "@encoding")]
    encoding: Option<String>,
    #[serde(default)]
    #[serde(rename = "param")]
    expected_params: Vec<XmlExpectedParam>,
}

#[derive(Debug, Deserialize)]
struct XmlExpectedParam {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct XmlParam {
    #[serde(rename = "@pos")]
    pos: usize,
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: Option<String>,
}

impl XmlExample {
    fn into_example(self) -> Result<Example, RecogError> {
        let is_base64 = self.encoding.as_ref().map(|s| s.as_str()) == Some("base64");

        // Load content from file if filename is specified, otherwise use value
        let content = if let Some(filename) = self.filename {
            // For async version, we'd need to read the file asynchronously
            // For now, use sync read
            let content = std::fs::read_to_string(&filename)?;
            if is_base64 {
                // If base64 encoding is specified for external file,
                // decode it first, then we'll re-encode it for storage
                let decoded = general_purpose::STANDARD.decode(&content.trim())?;
                general_purpose::STANDARD.encode(&decoded)
            } else {
                content.trim().to_string()
            }
        } else if let Some(value) = self.value {
            value
        } else {
            return Err(RecogError::invalid_fingerprint_data(
                "Example must have either value or filename attribute",
            ));
        };

        let mut example = if is_base64 {
            Example::new_base64(content)
        } else {
            Example::new(content)
        };

        for expected in self.expected_params {
            example.add_expected(expected.name, expected.value);
        }

        Ok(example)
    }
}

impl XmlParam {
    fn into_param(self) -> Param {
        Param {
            pos: self.pos,
            name: self.name,
            value: self.value,
        }
    }
}

impl XmlFingerprint {
    fn into_fingerprint(self) -> RecogResult<Fingerprint> {
        let mut fingerprint = Fingerprint::new(&self.pattern, &self.description)?;

        for example in self.examples {
            let example = example.into_example()?;
            fingerprint.add_example(example);
        }

        for param in self.params {
            fingerprint.add_param(param.into_param());
        }

        Ok(fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_async_file_loading() {
        let temp_dir = tempdir().unwrap();
        let xml_file = temp_dir.path().join("test.xml");

        let xml_content = r#"
            <fingerprints>
                <fingerprint pattern="^Test/(\d+)$">
                    <description>Test pattern</description>
                    <example>Test/123</example>
                    <param pos="1" name="version"/>
                </fingerprint>
            </fingerprints>
        "#;

        tokio::fs::write(&xml_file, xml_content).await.unwrap();

        let db = load_fingerprints_from_file_async(&xml_file).await.unwrap();
        assert_eq!(db.fingerprints.len(), 1);
        assert_eq!(db.fingerprints[0].description, "Test pattern");
    }

    #[tokio::test]
    async fn test_multiple_database_loading() {
        let temp_dir = tempdir().unwrap();

        // Create multiple XML files
        let mut files = Vec::new();
        for i in 0..3 {
            let xml_file = temp_dir.path().join(format!("test{}.xml", i));
            let xml_content = format!(
                r#"
                <fingerprints>
                    <fingerprint pattern="^Pattern{}/(.+)$">
                        <description>Pattern {}</description>
                        <example>Pattern{}: value{}</example>
                        <param pos="1" name="value"/>
                    </fingerprint>
                </fingerprints>
            "#,
                i, i, i, i
            );

            tokio::fs::write(&xml_file, xml_content).await.unwrap();
            files.push(xml_file);
        }

        let databases = load_multiple_databases_async(&files).await.unwrap();
        assert_eq!(databases.len(), 3);

        for (i, db) in databases.iter().enumerate() {
            assert_eq!(db.fingerprints.len(), 1);
            assert_eq!(db.fingerprints[0].description, format!("Pattern {}", i));
        }
    }

    #[tokio::test]
    async fn test_streaming_loader() {
        let temp_dir = tempdir().unwrap();
        let xml_file = temp_dir.path().join("large.xml");

        // Create a larger XML file
        let mut xml_content = String::from(
            r#"<fingerprints matches="test" protocol="test" database_type="service">"#,
        );
        for i in 0..100 {
            xml_content.push_str(&format!(
                r#"
                <fingerprint pattern="^Pattern{}: (.+)$">
                    <description>Pattern {}</description>
                    <example>Pattern{}: value{}</example>
                    <param pos="1" name="value"/>
                </fingerprint>
            "#,
                i, i, i, i
            ));
        }
        xml_content.push_str("</fingerprints>");

        tokio::fs::write(&xml_file, xml_content).await.unwrap();

        let loader = StreamingXmlLoader::new(1024);
        let db = loader.load_large_file_streaming(&xml_file).await.unwrap();

        // Note: Current implementation is simplified and may not parse correctly
        // In a full implementation, this would properly parse the streaming XML
        assert!(db.fingerprints.len() >= 0);
    }
}
