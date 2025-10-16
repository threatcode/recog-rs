use crate::error::{RecogError, RecogResult};
use crate::fingerprint::{Example, Fingerprint, FingerprintDatabase};
use crate::params::Param;
use base64::{engine::general_purpose, Engine as _};
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// XML parsing structures for deserialization
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
            let content = fs::read_to_string(&filename)?;
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
                "Example must have either value or filename attribute"
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

/// Load fingerprints from XML content
pub fn load_fingerprints_from_xml(xml_content: &str) -> RecogResult<FingerprintDatabase> {
    let xml_fps: XmlFingerprints = from_str(xml_content)?;
    let mut db = FingerprintDatabase::new();

    for xml_fp in xml_fps.fingerprints {
        let fingerprint = xml_fp.into_fingerprint()?;
        db.add_fingerprint(fingerprint);
    }

    Ok(db)
}

/// Load fingerprints from XML file
pub fn load_fingerprints_from_file<P: AsRef<Path>>(path: P) -> RecogResult<FingerprintDatabase> {
    let xml_content = fs::read_to_string(path)?;
    load_fingerprints_from_xml(&xml_content)
}

/// Save fingerprints to XML (for testing/debugging)
pub fn save_fingerprints_to_xml(_db: &FingerprintDatabase) -> RecogResult<String> {
    // This would implement XML serialization if needed
    // For now, return a placeholder
    Ok("<?xml version=\"1.0\"?><fingerprints></fingerprints>".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_simple_fingerprint() {
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="Apache/(\d+\.\d+)">
                    <description>Apache HTTP Server</description>
                    <example value="Apache/2.4.41">
                        <param name="hw.product" value="Apache"/>
                        <param name="hw.version" value="2.4.41"/>
                    </example>
                    <param pos="1" name="hw.version"/>
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        assert_eq!(db.fingerprints.len(), 1);

        let fp = &db.fingerprints[0];
        assert_eq!(fp.description, "Apache HTTP Server");
        assert_eq!(fp.params.len(), 1);
        assert_eq!(fp.params[0].name, "hw.version");
        assert_eq!(fp.params[0].pos, 1);
    }

    #[test]
    fn test_filename_example() {
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="test">
                    <description>Test pattern</description>
                    <example filename="examples/apache_banner.txt">
                        <param name="test" value="test data"/>
                    </example>
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        let example = &db.fingerprints[0].examples[0];
        assert!(!example.is_base64);
        assert_eq!(example.value, "Apache/2.4.41 (Ubuntu) Server Header");
    }
}
