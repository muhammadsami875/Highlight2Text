use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use crate::process::ProcessRunner;
use std::path::PathBuf;
use std::time::Duration;

const INPUT_FORMATS: &[&str] = &[
    "docx", "doc", "odt", "rtf", "xlsx", "xls", "ods", "pptx", "ppt", "odp", "html", "txt",
];

const OUTPUT_FORMATS: &[&str] = &[
    "pdf", "docx", "html", "txt", "odt", "xlsx", "csv", "pptx",
];

fn find_soffice() -> Option<PathBuf> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\LibreOffice\program\soffice.exe".into(),
            r"C:\Program Files (x86)\LibreOffice\program\soffice.exe".into(),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/Applications/LibreOffice.app/Contents/MacOS/soffice"),
            PathBuf::from("/opt/homebrew/bin/soffice"),
            PathBuf::from("/usr/local/bin/soffice"),
        ]
    } else {
        vec![
            PathBuf::from("/usr/bin/soffice"),
            PathBuf::from("/usr/bin/libreoffice"),
            PathBuf::from("/usr/local/bin/soffice"),
            PathBuf::from("/snap/bin/libreoffice"),
        ]
    };

    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }

    if let Ok(output) = ProcessRunner::new(PathBuf::from("which"))
        .arg("soffice")
        .timeout(Duration::from_secs(5))
        .run()
    {
        if output.success() {
            let path = output.stdout.trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    None
}

pub struct LibreOfficeConverter {
    manifest: ConverterManifest,
    soffice_path: Option<PathBuf>,
}

impl LibreOfficeConverter {
    pub fn new() -> Self {
        let soffice_path = find_soffice();
        Self {
            manifest: ConverterManifest {
                id: "libreoffice".to_string(),
                name: "LibreOffice Document Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::External {
                    executable: "soffice".to_string(),
                },
                license: "LGPL-3.0 (external process)".to_string(),
                input_formats: INPUT_FORMATS.iter().map(|s| s.to_string()).collect(),
                output_formats: OUTPUT_FORMATS.iter().map(|s| s.to_string()).collect(),
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 50,
                capabilities: vec!["document_conversion".into(), "external_engine".into()],
            },
            soffice_path,
        }
    }
}

impl Converter for LibreOfficeConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        self.soffice_path.is_some()
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        if from == to {
            return false;
        }
        INPUT_FORMATS.contains(&from) && OUTPUT_FORMATS.contains(&to)
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let soffice = self
            .soffice_path
            .as_ref()
            .ok_or_else(|| ConversionError::missing_engine("libreoffice"))?;

        let output_dir = job_dir.join("output");
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        let temp_dir = job_dir.join("lo_work");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| ConversionError::io_error(&format!("Cannot create temp dir: {}", e)))?;

        let user_profile = temp_dir.join("profile");
        std::fs::create_dir_all(&user_profile).map_err(|e| {
            ConversionError::io_error(&format!("Cannot create LO profile dir: {}", e))
        })?;

        let profile_url = format!(
            "file://{}",
            user_profile.to_string_lossy().replace('\\', "/")
        );

        let result = ProcessRunner::new(soffice.clone())
            .arg("--headless")
            .arg("--norestore")
            .arg("--nologo")
            .arg(format!("-env:UserInstallation={}", profile_url))
            .arg("--convert-to")
            .arg(&request.output_format)
            .arg("--outdir")
            .arg(output_dir.to_string_lossy().to_string())
            .arg(request.input_path.to_string_lossy().to_string())
            .timeout(Duration::from_secs(120))
            .run()
            .map_err(|e| ConversionError::engine_failure("libreoffice", &e))?;

        if result.timed_out {
            return Err(ConversionError::timeout());
        }

        if !result.success() {
            return Err(ConversionError::engine_failure(
                "libreoffice",
                &format!(
                    "exit code {}: {}",
                    result.exit_code,
                    result.stderr.lines().take(5).collect::<Vec<_>>().join("\n")
                ),
            ));
        }

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = output_dir.join(format!("{}.{}", stem, request.output_format));

        if !output_path.exists() {
            let mut found = None;
            if let Ok(entries) = std::fs::read_dir(&output_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e == request.output_format)
                        .unwrap_or(false)
                    {
                        found = Some(path);
                        break;
                    }
                }
            }

            match found {
                Some(path) => Ok(path),
                None => Err(ConversionError::engine_failure(
                    "libreoffice",
                    "LibreOffice did not produce an output file",
                )),
            }
        } else {
            Ok(output_path)
        }
    }
}
