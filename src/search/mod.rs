use crate::utils::remove_accents;
use lopdf::Document as PdfDoc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: i64,
    pub file_name: String,
    pub file_path: String,
    pub file_type: String,
    pub page_number: i32,
    pub snippet: String,
    pub indexed_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IndexProgress {
    pub is_running: bool,
    pub current_file: String,
    pub processed_files: usize,
    pub total_files: usize,
    pub percent: usize,
    pub message: String,
}

/// Trích xuất nội dung văn bản từ file Word (.docx) siêu tốc mà không cần MS Word
pub fn extract_text_from_docx<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut xml_content = String::new();
    if let Ok(mut doc_xml) = archive.by_name("word/document.xml") {
        doc_xml.read_to_string(&mut xml_content).map_err(|e| e.to_string())?;
    } else {
        return Err("Không tìm thấy word/document.xml trong file docx".into());
    }

    // Trích xuất chuỗi trong các tag <w:t>...</w:t>
    let mut result = String::with_capacity(xml_content.len() / 3);
    let mut in_wt = false;
    let mut in_wp = false;
    let mut chars = xml_content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            let mut tag = String::new();
            while let Some(&next_c) = chars.peek() {
                chars.next();
                if next_c == '>' {
                    break;
                }
                tag.push(next_c);
            }
            if tag.starts_with("w:t") {
                in_wt = true;
            } else if tag.starts_with("/w:t") {
                in_wt = false;
            } else if tag.starts_with("w:p") {
                in_wp = true;
            } else if tag.starts_with("/w:p") {
                if in_wp {
                    result.push('\n');
                    in_wp = false;
                }
            }
        } else if in_wt {
            result.push(c);
        }
    }

    Ok(result)
}

/// Trích xuất nội dung văn bản trực tiếp từ PDF theo từng trang (Native lopdf)
pub fn extract_text_from_pdf<P: AsRef<Path>>(path: P) -> Result<Vec<(i32, String)>, String> {
    let doc = PdfDoc::load(path).map_err(|e| e.to_string())?;
    let pages = doc.get_pages();
    let mut results = Vec::with_capacity(pages.len());

    for (page_num, _) in pages.iter() {
        let text = doc.extract_text(&[*page_num]).unwrap_or_default();
        let cleaned = text.trim();
        if !cleaned.is_empty() {
            results.push((*page_num as i32, cleaned.to_string()));
        }
    }

    Ok(results)
}

/// Tạo trích đoạn (snippet) quanh từ khóa tìm kiếm
fn create_snippet(content: &str, keyword: &str) -> String {
    let lower_content = content.to_lowercase();
    let lower_keyword = keyword.to_lowercase();

    if let Some(pos) = lower_content.find(&lower_keyword) {
        let start = pos.saturating_sub(60);
        let end = (pos + keyword.len() + 100).min(content.len());
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < content.len() { "..." } else { "" };
        format!("{}{}{}", prefix, &content[start..end].replace('\n', " "), suffix)
    } else {
        content.chars().take(120).collect::<String>().replace('\n', " ") + "..."
    }
}

/// Tìm kiếm toàn văn trong SQLite (dùng kết hợp FTS5 và không dấu)
pub fn search_documents(conn: &Connection, query: &str) -> rusqlite::Result<Vec<SearchResultItem>> {
    let query_clean = query.trim();
    if query_clean.is_empty() {
        return Ok(Vec::new());
    }

    let query_no_accent = remove_accents(query_clean);
    let like_pattern = format!("%{}%", query_clean);
    let like_no_accent = format!("%{}%", query_no_accent);

    let mut stmt = conn.prepare(
        "
        SELECT id, file_name, file_path, file_type, page_number, content, indexed_at
        FROM documents
        WHERE content LIKE ?1 OR content_no_accent LIKE ?2 OR file_name LIKE ?1
        ORDER BY file_name ASC, page_number ASC
        LIMIT 500
        ",
    )?;

    let rows = stmt.query_map(params![like_pattern, like_no_accent], |row| {
        let content: String = row.get(5)?;
        let snippet = create_snippet(&content, query_clean);
        Ok(SearchResultItem {
            id: row.get(0)?,
            file_name: row.get(1)?,
            file_path: row.get(2)?,
            file_type: row.get(3)?,
            page_number: row.get(4)?,
            snippet,
            indexed_at: row.get(6)?,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }

    Ok(results)
}
