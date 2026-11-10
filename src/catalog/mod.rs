use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogRow {
    pub id: Option<i64>,
    pub stt: i32,
    pub so_ky_hieu: String,
    pub ngay_ban_hanh: String,
    pub co_quan_ban_hanh: String,
    pub trich_yeu: String,
    pub to_so: String,
    pub do_mat: String,
    pub ghi_chu: String,
}

/// Trích xuất thông tin hành chính từ trang văn bản đầu tiên
pub fn extract_document_info(text: &str) -> (String, String, String, String, String) {
    let text_clean = text.replace('\r', "");
    let lines: Vec<&str> = text_clean.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    // 1. Nhận diện loại văn bản
    let mut loai_van_ban = "Công văn".to_string();
    let loai_list = [
        "NGHỊ QUYẾT", "QUYẾT ĐỊNH", "CHỈ THỊ", "KẾ HOẠCH", "THÔNG BÁO", 
        "TỜ TRÌNH", "BÁO CÁO", "BIÊN BẢN", "KẾT LUẬN", "CÔNG VĂN", "HƯỚNG DẪN"
    ];
    for line in &lines {
        let upper = line.to_uppercase();
        for &loai in &loai_list {
            if upper.contains(loai) {
                loai_van_ban = loai.to_string();
                break;
            }
        }
    }

    // 2. Trích xuất số và ký hiệu (VD: Số: 123/QĐ-UBND, Số 45/TB-CAT)
    let re_so = Regex::new(r"(?i)(?:Số|Số:)\s*([0-9]+[0-9a-zA-Z\/\-_]+)").unwrap();
    let mut so_ky_hieu = String::new();
    if let Some(caps) = re_so.captures(&text_clean) {
        if let Some(m) = caps.get(1) {
            so_ky_hieu = m.as_str().trim().to_string();
        }
    }

    // 3. Trích xuất ngày ban hành (VD: ngày 15 tháng 06 năm 2026)
    let re_ngay = Regex::new(r"(?i)(?:ngày|Ngày)\s*(\d{1,2})\s*tháng\s*(\d{1,2})\s*năm\s*(\d{4})").unwrap();
    let mut ngay_ban_hanh = String::new();
    if let Some(caps) = re_ngay.captures(&text_clean) {
        let d = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let m = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let y = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        ngay_ban_hanh = format!("{:0>2}/{:0>2}/{}", d, m, y);
    }

    // 4. Cơ quan ban hành (thường nằm ở dòng 1-3 bên trái)
    let mut co_quan = String::new();
    for line in lines.iter().take(4) {
        let upper = line.to_uppercase();
        if upper.contains("ỦY BAN") || upper.contains("CÔNG AN") || upper.contains("THANH TRA") || upper.contains("BỘ ") || upper.contains("SỞ ") {
            co_quan = line.to_string();
            break;
        }
    }
    if co_quan.is_empty() && !lines.is_empty() {
        co_quan = lines[0].to_string();
    }

    // 5. Trích yếu (sau V/v: hoặc câu tóm tắt)
    let re_trich_yeu = Regex::new(r"(?i)(?:V/v|Về việc)\s*[:：]?\s*([^\n\r]+)").unwrap();
    let mut trich_yeu = String::new();
    if let Some(caps) = re_trich_yeu.captures(&text_clean) {
        if let Some(m) = caps.get(1) {
            trich_yeu = m.as_str().trim().to_string();
        }
    }
    if trich_yeu.is_empty() {
        trich_yeu = format!("{} về công tác chuyên môn", loai_van_ban);
    }

    (loai_van_ban, so_ky_hieu, ngay_ban_hanh, co_quan, trich_yeu)
}

/// Xuất danh mục văn bản hồ sơ sang file Word (.docx) chuẩn Mẫu số 03
pub fn export_catalog_to_docx<P: AsRef<Path>>(output_path: P, rows: &[CatalogRow], tieu_de: &str) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#)?;

    // 2. _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#)?;

    // 3. word/document.xml
    zip.start_file("word/document.xml", options)?;
    let mut doc_xml = String::new();
    doc_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:jc w:val="center"/></w:pPr>
      <w:r><w:rPr><w:b/><w:sz w:val="32"/><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/></w:rPr>
        <w:t>MỤC LỤC VĂN BẢN TRONG HỒ SƠ</w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:pPr><w:jc w:val="center"/></w:pPr>
      <w:r><w:rPr><w:i/><w:sz w:val="24"/><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/></w:rPr>
        <w:t>"#);
    doc_xml.push_str(&quick_xml_escape(tieu_de));
    doc_xml.push_str(r#"</w:t>
      </w:r>
    </w:p>
    <w:tbl>
      <w:tblPr>
        <w:tblW w:w="0" w:type="auto"/>
        <w:tblBorders>
          <w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/>
          <w:left w:val="single" w:sz="4" w:space="0" w:color="000000"/>
          <w:bottom w:val="single" w:sz="4" w:space="0" w:color="000000"/>
          <w:right w:val="single" w:sz="4" w:space="0" w:color="000000"/>
          <w:insideH w:val="single" w:sz="4" w:space="0" w:color="000000"/>
          <w:insideV w:val="single" w:sz="4" w:space="0" w:color="000000"/>
        </w:tblBorders>
      </w:tblPr>
      <w:tr>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>STT</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Số, ký hiệu</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Ngày ban hành</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Cơ quan ban hành</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Trích yếu nội dung</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Tờ số</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Độ mật</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>Ghi chú</w:t></w:r></w:p></w:tc>
      </w:tr>"#);

    for (idx, row) in rows.iter().enumerate() {
        let stt = if row.stt > 0 { row.stt } else { (idx + 1) as i32 };
        doc_xml.push_str(&format!(
            r#"<w:tr>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:rPr><w:rFonts w:ascii="Times New Roman"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
      </w:tr>"#,
            stt,
            quick_xml_escape(&row.so_ky_hieu),
            quick_xml_escape(&row.ngay_ban_hanh),
            quick_xml_escape(&row.co_quan_ban_hanh),
            quick_xml_escape(&row.trich_yeu),
            quick_xml_escape(&row.to_so),
            quick_xml_escape(&row.do_mat),
            quick_xml_escape(&row.ghi_chu)
        ));
    }

    doc_xml.push_str(r#"
    </w:tbl>
    <w:sectPr>
      <w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/>
      <w:pgMar w:top="1134" w:right="1134" w:bottom="1134" w:left="1417"/>
    </w:sectPr>
  </w:body>
</w:document>"#);

    zip.write_all(doc_xml.as_bytes())?;
    zip.finish()?;

    Ok(())
}

fn quick_xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
