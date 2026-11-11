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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedDoc {
    pub file_name: String,
    pub file_path: String,
    pub file_type: String,
    pub total_pages: i32,
    pub indexed_at: String,
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
    if keyword.is_empty() {
        return content.chars().take(120).collect::<String>().replace('\n', " ") + "...";
    }

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

/// Trích xuất văn bản từ tag XML (như <a:t>, <w:t>, <t>) và xuống dòng khi gặp block tag
fn extract_xml_tag_text(xml: &str, text_tag: &str, block_tag: &str) -> String {
    let mut result = String::with_capacity(xml.len() / 4);
    let mut in_target = false;
    let mut chars = xml.chars().peekable();

    let close_text_tag = format!("/{}", text_tag);
    let close_block_tag = format!("/{}", block_tag);

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
            if tag == text_tag || tag.starts_with(&format!("{} ", text_tag)) {
                in_target = true;
            } else if tag == close_text_tag || tag.starts_with(&close_text_tag) {
                in_target = false;
                result.push(' ');
            } else if tag == close_block_tag || tag.starts_with(&close_block_tag) {
                result.push('\n');
            }
        } else if in_target {
            result.push(c);
        }
    }

    result
}

/// Trích xuất nội dung văn bản từ PowerPoint (.pptx) theo từng slide
pub fn extract_text_from_pptx<P: AsRef<Path>>(path: P) -> Result<Vec<(i32, String)>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut slide_names = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_names.push(name);
            }
        }
    }

    slide_names.sort_by_key(|name| {
        let num_part = name
            .trim_start_matches("ppt/slides/slide")
            .trim_end_matches(".xml");
        num_part.parse::<i32>().unwrap_or(9999)
    });

    let mut results = Vec::new();
    let mut slide_index = 1;

    for slide_name in slide_names {
        if let Ok(mut entry) = archive.by_name(&slide_name) {
            let mut xml_content = String::new();
            if entry.read_to_string(&mut xml_content).is_ok() {
                let text = extract_xml_tag_text(&xml_content, "a:t", "a:p");
                let cleaned = text.trim();
                if !cleaned.is_empty() {
                    results.push((slide_index, cleaned.to_string()));
                }
            }
        }
        slide_index += 1;
    }

    if results.is_empty() {
        Ok(vec![(1, String::new())])
    } else {
        Ok(results)
    }
}

/// Trích xuất nội dung văn bản từ bảng tính Excel (.xlsx)
pub fn extract_text_from_xlsx<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut all_text = String::new();

    // 1. Trích xuất từ xl/sharedStrings.xml (chứa chuỗi văn bản của các ô)
    if let Ok(mut ss_entry) = archive.by_name("xl/sharedStrings.xml") {
        let mut xml_content = String::new();
        if ss_entry.read_to_string(&mut xml_content).is_ok() {
            let text = extract_xml_tag_text(&xml_content, "t", "si");
            all_text.push_str(&text);
        }
    }

    // 2. Trích xuất thêm từ sheet xml
    let mut sheet_names = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            if name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml") {
                sheet_names.push(name);
            }
        }
    }

    for sheet_name in sheet_names {
        if let Ok(mut entry) = archive.by_name(&sheet_name) {
            let mut xml_content = String::new();
            if entry.read_to_string(&mut xml_content).is_ok() {
                let text = extract_xml_tag_text(&xml_content, "v", "row");
                if !text.trim().is_empty() {
                    all_text.push('\n');
                    all_text.push_str(&text);
                }
            }
        }
    }

    Ok(all_text)
}

/// Trích xuất file văn bản thuần (.txt, .md, .csv, .log, .rtf)
pub fn extract_text_from_plain_text<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let max_len = bytes.len().min(10 * 1024 * 1024);
    let slice = &bytes[..max_len];
    Ok(String::from_utf8_lossy(slice).to_string())
}

/// Tìm kiếm văn bản trong SQLite (hỗ trợ lọc đa loại file và tìm kiếm theo thời gian thực)
pub fn search_documents(conn: &Connection, query: &str, file_type: &str) -> rusqlite::Result<Vec<SearchResultItem>> {
    let query_clean = query.trim();
    let query_no_accent = remove_accents(query_clean);
    let like_pattern = format!("%{}%", query_clean);
    let like_no_accent = format!("%{}%", query_no_accent);

    let is_all = file_type.is_empty() || file_type.eq_ignore_ascii_case("all");
    let filter_list = format!(",{},", file_type.to_uppercase());
    let is_all_int = if is_all { 1 } else { 0 };

    let mut stmt = conn.prepare(
        "
        SELECT id, file_name, file_path, file_type, page_number, content, indexed_at
        FROM documents
        WHERE (?1 = 1 OR instr(?2, ',' || file_type || ',') > 0)
          AND (?3 = '' OR content LIKE ?4 OR content_no_accent LIKE ?5 OR file_name LIKE ?4)
        ORDER BY file_name ASC, page_number ASC
        LIMIT 300
        ",
    )?;

    let rows = stmt.query_map(params![is_all_int, filter_list, query_clean, like_pattern, like_no_accent], |row| {
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

/// Lấy danh sách tài liệu đã lập chỉ mục trong hệ thống (hỗ trợ lọc đa loại file)
pub fn get_scanned_files(conn: &Connection, file_type: &str) -> rusqlite::Result<Vec<ScannedDoc>> {
    let is_all = file_type.is_empty() || file_type.eq_ignore_ascii_case("all");
    let filter_list = format!(",{},", file_type.to_uppercase());
    let is_all_int = if is_all { 1 } else { 0 };

    let mut stmt = conn.prepare(
        "
        SELECT file_name, file_path, file_type, COUNT(DISTINCT page_number) as total_pages, MAX(indexed_at) as indexed_at
        FROM documents
        WHERE (?1 = 1 OR instr(?2, ',' || file_type || ',') > 0)
        GROUP BY file_name, file_path, file_type
        ORDER BY MAX(id) DESC
        LIMIT 500
        ",
    )?;

    let rows = stmt.query_map(params![is_all_int, filter_list], |row| {
        Ok(ScannedDoc {
            file_name: row.get(0)?,
            file_path: row.get(1)?,
            file_type: row.get(2)?,
            total_pages: row.get(3)?,
            indexed_at: row.get(4)?,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }

    Ok(results)
}
