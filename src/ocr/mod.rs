use docx_rs::*;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageOcrResult {
    pub page_number: usize,
    pub filename: String,
    pub text: String,
    pub word_count: usize,
    pub char_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOcrResponse {
    pub success: bool,
    pub total_pages: usize,
    pub total_words: usize,
    pub total_chars: usize,
    pub pages: Vec<PageOcrResult>,
    pub combined_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDocxRequest {
    pub title: String,
    pub pages: Vec<PageOcrResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrStatus {
    pub available: bool,
    pub engine: String,
    pub languages: Vec<String>,
    pub exe_path: Option<String>,
}

pub fn find_tesseract_bin() -> Option<(PathBuf, PathBuf)> {
    // 1. Next to current running executable: tools/tesseract/tesseract.exe
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let exe = parent.join("tools").join("tesseract").join("tesseract.exe");
            let tessdata = parent.join("tools").join("tesseract").join("tessdata");
            if exe.exists() {
                return Some((exe, tessdata));
            }
        }
    }
    // 2. Relative to working directory: tools/tesseract/tesseract.exe
    let cwd_exe = PathBuf::from("tools").join("tesseract").join("tesseract.exe");
    let cwd_tessdata = PathBuf::from("tools").join("tesseract").join("tessdata");
    if cwd_exe.exists() {
        return Some((cwd_exe, cwd_tessdata));
    }
    // 3. In portable pack directory
    let portable_exe = PathBuf::from(r"E:\port to rust\CYA_SUITE_PORTABLE_v1.0\tools\tesseract\tesseract.exe");
    let portable_tessdata = PathBuf::from(r"E:\port to rust\CYA_SUITE_PORTABLE_v1.0\tools\tesseract\tessdata");
    if portable_exe.exists() {
        return Some((portable_exe, portable_tessdata));
    }
    // 4. In system PATH
    if let Ok(output) = std::process::Command::new("tesseract").arg("--version").output() {
        if output.status.success() {
            return Some((PathBuf::from("tesseract"), PathBuf::from("tessdata")));
        }
    }
    None
}

pub fn check_ocr_status() -> OcrStatus {
    if let Some((exe, tessdata)) = find_tesseract_bin() {
        let mut langs = Vec::new();
        if tessdata.join("vie.traineddata").exists() {
            langs.push("vie (Tiếng Việt)".to_string());
        }
        if tessdata.join("eng.traineddata").exists() {
            langs.push("eng (Tiếng Anh)".to_string());
        }
        OcrStatus {
            available: true,
            engine: "Google Tesseract OCR 5 (LSTM Neural - 100% Offline)".to_string(),
            languages: langs,
            exe_path: Some(exe.to_string_lossy().to_string()),
        }
    } else {
        OcrStatus {
            available: false,
            engine: "Chưa cấu hình Tesseract".to_string(),
            languages: vec![],
            exe_path: None,
        }
    }
}

pub fn ocr_single_image(image_bytes: &[u8], lang: &str) -> Result<String, String> {
    let (tess_exe, tessdata_dir) = find_tesseract_bin()
        .ok_or_else(|| "Không tìm thấy bộ máy Tesseract OCR (tools/tesseract/tesseract.exe)".to_string())?;

    let temp_dir = std::env::temp_dir().join("cya_ocr");
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let file_id = uuid::Uuid::new_v4().to_string();
    let temp_input = temp_dir.join(format!("{}.img", file_id));
    let temp_output_base = temp_dir.join(format!("{}_out", file_id));
    let temp_output_txt = temp_dir.join(format!("{}_out.txt", file_id));

    std::fs::write(&temp_input, image_bytes).map_err(|e| format!("Lỗi ghi file tạm ảnh: {}", e))?;

    let mut cmd = std::process::Command::new(&tess_exe);
    cmd.arg(&temp_input)
       .arg(&temp_output_base)
       .arg("-l")
       .arg(if lang.is_empty() { "vie" } else { lang });

    if tessdata_dir.exists() {
        cmd.arg("--tessdata-dir").arg(&tessdata_dir);
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|e| format!("Lỗi thực thi Tesseract: {}", e))?;

    let text = if temp_output_txt.exists() {
        let content = std::fs::read_to_string(&temp_output_txt).unwrap_or_default();
        let _ = std::fs::remove_file(&temp_output_txt);
        content
    } else if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        let _ = std::fs::remove_file(&temp_input);
        return Err(format!("Lỗi nhận dạng ảnh: {}", err));
    };

    let _ = std::fs::remove_file(&temp_input);
    let _ = std::fs::remove_file(&temp_output_txt);

    Ok(text.trim().to_string())
}

pub fn generate_docx(title: &str, pages: &[PageOcrResult]) -> Result<Vec<u8>, String> {
    let mut file = Cursor::new(Vec::new());
    let mut docx = Docx::new();

    let display_title = if title.trim().is_empty() { "TÀI LIỆU SCAN OCR" } else { title.trim() };
    let title_para = Paragraph::new()
        .align(AlignmentType::Center)
        .add_run(Run::new().add_text(display_title).bold().size(32));
    docx = docx.add_paragraph(title_para);
    docx = docx.add_paragraph(Paragraph::new());

    for (idx, page) in pages.iter().enumerate() {
        if idx > 0 {
            let pb_para = Paragraph::new().page_break_before(true);
            docx = docx.add_paragraph(pb_para);
        }

        let page_hdr = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(format!("--- Trang {} ({}) ---", page.page_number, page.filename))
                    .bold()
                    .italic()
                    .size(20)
            );
        docx = docx.add_paragraph(page_hdr);
        docx = docx.add_paragraph(Paragraph::new());

        for line in page.text.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                docx = docx.add_paragraph(Paragraph::new());
            } else {
                let p = Paragraph::new()
                    .line_spacing(LineSpacing::new().line(276))
                    .add_run(
                        Run::new()
                            .add_text(line_trimmed)
                            .fonts(RunFonts::new().ascii("Times New Roman"))
                            .size(28)
                    );
                docx = docx.add_paragraph(p);
            }
        }
    }

    docx.build().pack(&mut file).map_err(|e| format!("Lỗi đóng gói file DOCX: {:?}", e))?;
    Ok(file.into_inner())
}
