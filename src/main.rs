mod admin_doc;
mod catalog;
mod db;
mod kpi;
mod ocr;
mod search;
mod state;
mod ui;
mod utils;
mod voice;

use axum::{
    extract::{Multipart, Query, State},
    http::header,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Local;
use search::{
    extract_text_from_docx, extract_text_from_pdf, extract_text_from_plain_text,
    extract_text_from_pptx, extract_text_from_xlsx, get_scanned_files, search_documents,
    ScannedDoc, SearchResultItem,
};
use serde::{Deserialize, Serialize};
use state::{AppState, IndexingStatus};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utils::remove_accents;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = PathBuf::from("cya_data");
    fs::create_dir_all(&data_dir)?;
    let temp_dir = data_dir.join("temp");
    fs::create_dir_all(&temp_dir)?;

    let db_path = data_dir.join("cya_suite.db");
    let conn = db::init_db(&db_path)?;
    let shared_state = Arc::new(AppState::new(conn, data_dir));

    let app = Router::new()
        .route("/", get(index_handler))
        // API Chọn thư mục & Ổ đĩa
        .route("/api/pick-folder", get(pick_folder_handler))
        .route("/api/browse-dir", get(browse_dir_handler))
        .route("/api/system-drives", get(get_system_drives_handler))
        // API Lập chỉ mục có tiến trình & nút dừng
        .route("/api/index/start", post(start_indexing_handler))
        .route("/api/index/stop", post(stop_indexing_handler))
        .route("/api/index/status", get(get_indexing_status_handler))
        .route("/api/scanned-files", get(get_scanned_files_handler))
        // API Tìm kiếm & Mở file/thư mục
        .route("/api/search", get(search_handler))
        .route("/api/open-file", get(open_file_handler))
        .route("/api/open-folder", get(open_folder_handler))
        // API Lập danh mục
        .route("/api/catalog/extract", post(extract_catalog_handler))
        .route("/api/catalog/export", post(export_catalog_handler))
        // API Voice Studio
        .route("/api/voice/voices", get(get_voices_handler))
        .route("/api/voice/synthesize", post(synthesize_voice_handler))
        .route("/api/voice/file/{name}", get(serve_temp_file_handler))
        // API KPI
        .route("/api/kpi/officers", get(get_officers_handler))
        .route("/api/kpi/tasks", get(get_tasks_handler))
        .route("/api/kpi/assignments", get(get_assignments_handler))
        .route("/api/kpi/assign", post(add_assignment_handler))
        .route("/api/kpi/summary", get(get_kpi_summary_handler))
        // API Scan OCR Văn bản
        .route("/api/ocr/status", get(ocr_status_handler))
        .route("/api/ocr/scan-batch", post(ocr_scan_batch_handler))
        .route("/api/ocr/export-docx", post(ocr_export_docx_handler))
        // API Soạn thảo & Biểu mẫu Văn bản Hành chính (Nghị định 30/2020/NĐ-CP)
        .route("/api/admin-doc/templates", get(get_admin_doc_templates_handler))
        .route("/api/admin-doc/export-docx", post(export_admin_docx_handler))
        // API Quản lý Tài nguyên & Hệ thống (Heartbeat, Auto-Kill Task & Thoát Ứng dụng)
        .route("/api/system/heartbeat", post(heartbeat_handler))
        .route("/api/system/shutdown", post(shutdown_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state.clone());

    let port = 8080;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://127.0.0.1:{}", port);

    println!("============================================================");
    println!("  CYA SUITE - HỆ THỐNG VĂN PHÒNG & TỰ ĐỘNG HÓA AI (RUST)");
    println!("============================================================");
    println!("  -> Máy chủ đang chạy tại: {}", url);
    println!("  -> Đang tự động mở trình duyệt...");

    let _ = webbrowser::open(&url);

    // Watchdog Task: Tự động phát hiện khi đóng toàn bộ tab trình duyệt (sau 15s không có heartbeat)
    // Tự động dừng mọi tác vụ quét nền và thoát hoàn toàn tiến trình để giải phóng 100% RAM & CPU
    let watchdog_state = shared_state.clone();
    tokio::spawn(async move {
        // Chờ 30 giây đầu cho trình duyệt khởi động và nạp trang
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let last = watchdog_state.last_heartbeat.load(Ordering::SeqCst);
            if last > 0 {
                let now = chrono::Utc::now().timestamp();
                if now - last > 15 {
                    println!("  -> [WATCHDOG] Phát hiện tất cả tab trình duyệt đã đóng (Timeout > 15s).");
                    println!("  -> Đang dừng tiến trình quét và giải phóng toàn bộ tài nguyên máy tính...");
                    watchdog_state.stop_flag.store(true, Ordering::SeqCst);
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    std::process::exit(0);
                }
            }
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(ui::INDEX_HTML)
}

/// Mở hộp thoại chọn thư mục chuẩn Windows (TopMost foreground)
async fn pick_folder_handler() -> Json<Option<String>> {
    let folder = tokio::task::spawn_blocking(utils::pick_folder_native)
        .await
        .unwrap_or(None);
    Json(folder)
}

#[derive(Serialize)]
struct DriveItem {
    label: String,
    path: String,
}

/// Lấy danh sách các ổ đĩa và thư mục tài liệu thông dụng
async fn get_system_drives_handler() -> Json<Vec<DriveItem>> {
    let mut items = Vec::new();
    items.push(DriveItem {
        label: "🚀 Toàn bộ máy (Tất cả ổ đĩa)".into(),
        path: "ALL".into(),
    });

    for d in utils::get_system_drives() {
        items.push(DriveItem {
            label: format!("Ổ {}", d),
            path: d,
        });
    }

    if let Ok(profile) = std::env::var("USERPROFILE") {
        let docs = format!("{}\\Documents", profile);
        if Path::new(&docs).exists() {
            items.push(DriveItem {
                label: "📁 Documents".into(),
                path: docs,
            });
        }
        let desktop = format!("{}\\Desktop", profile);
        if Path::new(&desktop).exists() {
            items.push(DriveItem {
                label: "🖥️ Desktop".into(),
                path: desktop,
            });
        }
        let downloads = format!("{}\\Downloads", profile);
        if Path::new(&downloads).exists() {
            items.push(DriveItem {
                label: "📥 Downloads".into(),
                path: downloads,
            });
        }
    }

    Json(items)
}

#[derive(Deserialize)]
struct BrowseDirReq {
    path: Option<String>,
}

#[derive(Serialize)]
struct DirEntryItem {
    name: String,
    path: String,
}

#[derive(Serialize)]
struct BrowseDirRes {
    current_path: String,
    parent_path: Option<String>,
    folders: Vec<DirEntryItem>,
}

/// Duyệt thư mục trực quan theo thời gian thực
async fn browse_dir_handler(Query(params): Query<BrowseDirReq>) -> Json<BrowseDirRes> {
    let current_path = params.path.unwrap_or_default().trim().to_string();

    if current_path.is_empty() {
        let drives = utils::get_system_drives();
        let folders = drives
            .into_iter()
            .map(|d| DirEntryItem {
                name: format!("Ổ đĩa {}", d),
                path: d,
            })
            .collect();
        return Json(BrowseDirRes {
            current_path: "".into(),
            parent_path: None,
            folders,
        });
    }

    let p = PathBuf::from(&current_path);
    let parent_path = p.parent().map(|parent| parent.to_string_lossy().to_string());

    let mut folders = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&p) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('$') && !name.starts_with('.') && !is_ignored_dir(&name) {
                        folders.push(DirEntryItem {
                            name,
                            path: entry.path().to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }
    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Json(BrowseDirRes {
        current_path,
        parent_path,
        folders,
    })
}

#[derive(Deserialize)]
struct IndexFolderReq {
    folder: String,
}

#[derive(Serialize)]
struct GenericRes {
    success: bool,
    message: String,
}

/// Kiểm tra thư mục hệ thống / rác cần bỏ qua để quét siêu tốc không bị đơ
fn is_ignored_dir(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "$recycle.bin"
            | "$winreagent"
            | "system volume information"
            | "recovery"
            | "windows"
            | "winsxs"
            | "program files"
            | "program files (x86)"
            | "node_modules"
            | ".git"
            | ".svn"
            | ".cache"
            | "target"
            | "vendor"
            | "appdata"
    )
}

/// Bắt đầu quét thư mục hoặc toàn bộ máy (Streaming Indexing tức thì)
async fn start_indexing_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IndexFolderReq>,
) -> Json<GenericRes> {
    let folder = payload.folder.trim().to_string();

    let targets: Vec<PathBuf> = if folder.eq_ignore_ascii_case("all")
        || folder.to_lowercase().contains("toàn bộ")
        || folder.is_empty()
    {
        utils::get_system_drives().into_iter().map(PathBuf::from).collect()
    } else {
        let p = PathBuf::from(&folder);
        if !p.exists() || !p.is_dir() {
            return Json(GenericRes {
                success: false,
                message: format!("Đường dẫn thư mục '{}' không tồn tại hoặc không hợp lệ!", folder),
            });
        }
        vec![p]
    };

    {
        let mut status = state.indexing_status.lock().unwrap();
        if status.is_running {
            return Json(GenericRes {
                success: false,
                message: "Hệ thống đang trong quá trình quét!".into(),
            });
        }
        status.is_running = true;
        status.stopped = false;
        status.percent = 0;
        status.scanned_count = 0;
        status.total_files = 0;
        status.message = "Đang khởi tạo trình quét dữ liệu tức thì...".into();
    }
    state.stop_flag.store(false, Ordering::SeqCst);

    let state_clone = state.clone();
    tokio::spawn(async move {
        let stop_flag = state_clone.stop_flag.clone();
        let status_arc = state_clone.indexing_status.clone();
        let db_arc = state_clone.db.clone();

        let mut total_indexed = 0;

        for target_dir in targets {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            {
                let mut status = status_arc.lock().unwrap();
                status.message = format!("Đang quét: {}...", target_dir.display());
            }

            let walker = WalkDir::new(&target_dir)
                .into_iter()
                .filter_entry(|e| {
                    if e.file_type().is_dir() && e.depth() > 0 {
                        if let Some(name) = e.file_name().to_str() {
                            if is_ignored_dir(name) {
                                return false;
                            }
                        }
                    }
                    true
                });

            for entry in walker.filter_map(|e| e.ok()) {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }

                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        let (doc_type, is_supported) = match ext_lower.as_str() {
                            "pdf" => ("PDF", true),
                            "docx" => ("WORD", true),
                            "xlsx" | "csv" => ("EXCEL", true),
                            "pptx" => ("PPT", true),
                            "txt" | "md" | "log" | "rtf" => ("TXT", true),
                            _ => ("", false),
                        };

                        if is_supported {
                            let file_name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                            let file_path_str = p.to_string_lossy().to_string();

                            total_indexed += 1;
                            {
                                let mut status = status_arc.lock().unwrap();
                                status.scanned_count = total_indexed;
                                status.current_file = file_name.clone();
                                status.message = format!("Đã quét {} tài liệu (Đang đọc: {})", total_indexed, file_name);
                            }

                            let now_str = Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
                            if doc_type == "WORD" {
                                if let Ok(text) = extract_text_from_docx(p) {
                                    if !text.trim().is_empty() {
                                        let text_no_acc = remove_accents(&text);
                                        let conn = db_arc.lock().unwrap();
                                        let _ = conn.execute("DELETE FROM documents WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute("DELETE FROM documents_fts WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute(
                                            "INSERT INTO documents (file_path, file_name, file_type, page_number, content, content_no_accent, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                                            rusqlite::params![file_path_str, file_name, "WORD", 1, text, text_no_acc, now_str],
                                        );
                                        let _ = conn.execute(
                                            "INSERT INTO documents_fts (file_path, file_name, file_type, page_number, content, content_no_accent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                            rusqlite::params![file_path_str, file_name, "WORD", 1, text, text_no_acc],
                                        );
                                    }
                                }
                            } else if doc_type == "PDF" {
                                if let Ok(pages) = extract_text_from_pdf(p) {
                                    if !pages.is_empty() {
                                        let conn = db_arc.lock().unwrap();
                                        let _ = conn.execute("DELETE FROM documents WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute("DELETE FROM documents_fts WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        for (page_no, text) in pages {
                                            let text_no_acc = remove_accents(&text);
                                            let _ = conn.execute(
                                                "INSERT INTO documents (file_path, file_name, file_type, page_number, content, content_no_accent, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                                                rusqlite::params![file_path_str, file_name, "PDF", page_no, text, text_no_acc, now_str],
                                            );
                                            let _ = conn.execute(
                                                "INSERT INTO documents_fts (file_path, file_name, file_type, page_number, content, content_no_accent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                                rusqlite::params![file_path_str, file_name, "PDF", page_no, text, text_no_acc],
                                            );
                                        }
                                    }
                                }
                            } else if doc_type == "PPT" {
                                if let Ok(slides) = extract_text_from_pptx(p) {
                                    if !slides.is_empty() {
                                        let conn = db_arc.lock().unwrap();
                                        let _ = conn.execute("DELETE FROM documents WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute("DELETE FROM documents_fts WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        for (slide_no, text) in slides {
                                            let text_no_acc = remove_accents(&text);
                                            let _ = conn.execute(
                                                "INSERT INTO documents (file_path, file_name, file_type, page_number, content, content_no_accent, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                                                rusqlite::params![file_path_str, file_name, "PPT", slide_no, text, text_no_acc, now_str],
                                            );
                                            let _ = conn.execute(
                                                "INSERT INTO documents_fts (file_path, file_name, file_type, page_number, content, content_no_accent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                                rusqlite::params![file_path_str, file_name, "PPT", slide_no, text, text_no_acc],
                                            );
                                        }
                                    }
                                }
                            } else if doc_type == "EXCEL" {
                                let text_res = if ext_lower == "csv" {
                                    extract_text_from_plain_text(p)
                                } else {
                                    extract_text_from_xlsx(p)
                                };
                                if let Ok(text) = text_res {
                                    if !text.trim().is_empty() {
                                        let text_no_acc = remove_accents(&text);
                                        let conn = db_arc.lock().unwrap();
                                        let _ = conn.execute("DELETE FROM documents WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute("DELETE FROM documents_fts WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute(
                                            "INSERT INTO documents (file_path, file_name, file_type, page_number, content, content_no_accent, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                                            rusqlite::params![file_path_str, file_name, "EXCEL", 1, text, text_no_acc, now_str],
                                        );
                                        let _ = conn.execute(
                                            "INSERT INTO documents_fts (file_path, file_name, file_type, page_number, content, content_no_accent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                            rusqlite::params![file_path_str, file_name, "EXCEL", 1, text, text_no_acc],
                                        );
                                    }
                                }
                            } else if doc_type == "TXT" {
                                if let Ok(text) = extract_text_from_plain_text(p) {
                                    if !text.trim().is_empty() {
                                        let text_no_acc = remove_accents(&text);
                                        let conn = db_arc.lock().unwrap();
                                        let _ = conn.execute("DELETE FROM documents WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute("DELETE FROM documents_fts WHERE file_path = ?1", rusqlite::params![file_path_str]);
                                        let _ = conn.execute(
                                            "INSERT INTO documents (file_path, file_name, file_type, page_number, content, content_no_accent, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                                            rusqlite::params![file_path_str, file_name, "TXT", 1, text, text_no_acc, now_str],
                                        );
                                        let _ = conn.execute(
                                            "INSERT INTO documents_fts (file_path, file_name, file_type, page_number, content, content_no_accent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                            rusqlite::params![file_path_str, file_name, "TXT", 1, text, text_no_acc],
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut status = status_arc.lock().unwrap();
        status.is_running = false;
        status.percent = 100;
        status.scanned_count = total_indexed;
        if status.stopped {
            status.message = format!("Đã dừng quét! Đã lập chỉ mục {} tài liệu.", total_indexed);
        } else {
            status.message = format!("Hoàn tất! Đã lập chỉ mục thành công {} tài liệu.", total_indexed);
        }
    });

    Json(GenericRes {
        success: true,
        message: "Đã bắt đầu tiến trình quét dữ liệu tức thì...".into(),
    })
}

/// Dừng quá trình quét
async fn stop_indexing_handler(State(state): State<Arc<AppState>>) -> Json<GenericRes> {
    state.stop_flag.store(true, Ordering::SeqCst);
    let mut status = state.indexing_status.lock().unwrap();
    status.stopped = true;
    status.message = "Đang gửi tín hiệu dừng quét...".into();

    Json(GenericRes {
        success: true,
        message: "Đã gửi lệnh dừng quét.".into(),
    })
}

/// Lấy tiến độ quét hiện tại
async fn get_indexing_status_handler(State(state): State<Arc<AppState>>) -> Json<IndexingStatus> {
    let status = state.indexing_status.lock().unwrap().clone();
    Json(status)
}

#[derive(Deserialize)]
struct FilterReq {
    file_type: Option<String>,
}

/// Lấy danh sách toàn bộ các file đã được quét trong database
async fn get_scanned_files_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FilterReq>,
) -> Json<Vec<ScannedDoc>> {
    let file_type = params.file_type.unwrap_or_else(|| "ALL".into());
    let conn = state.db.lock().unwrap();
    let docs = get_scanned_files(&conn, &file_type).unwrap_or_default();
    Json(docs)
}

#[derive(Deserialize)]
struct SearchReq {
    q: Option<String>,
    file_type: Option<String>,
}

/// Tìm kiếm toàn văn tức thời theo từng ký tự (Live Search)
async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchReq>,
) -> Json<Vec<SearchResultItem>> {
    let q = params.q.unwrap_or_default();
    let file_type = params.file_type.unwrap_or_else(|| "ALL".into());
    let conn = state.db.lock().unwrap();
    let results = search_documents(&conn, &q, &file_type).unwrap_or_default();
    Json(results)
}

#[derive(Deserialize)]
struct OpenFileReq {
    path: String,
}

async fn open_file_handler(Query(params): Query<OpenFileReq>) -> Json<bool> {
    let _ = utils::open_file_or_folder(&params.path);
    Json(true)
}

async fn open_folder_handler(Query(params): Query<OpenFileReq>) -> Json<bool> {
    let _ = utils::open_containing_folder(&params.path);
    Json(true)
}

#[derive(Deserialize)]
struct ExtractCatalogReq {
    file_path: String,
}

#[derive(Serialize)]
struct ExtractCatalogRes {
    success: bool,
    loai_van_ban: String,
    so_ky_hieu: String,
    ngay_ban_hanh: String,
    co_quan_ban_hanh: String,
    trich_yeu: String,
    total_pages: usize,
    error: Option<String>,
}

async fn extract_catalog_handler(Json(payload): Json<ExtractCatalogReq>) -> Json<ExtractCatalogRes> {
    let path = PathBuf::from(&payload.file_path);
    if !path.exists() {
        return Json(ExtractCatalogRes {
            success: false,
            loai_van_ban: "".into(),
            so_ky_hieu: "".into(),
            ngay_ban_hanh: "".into(),
            co_quan_ban_hanh: "".into(),
            trich_yeu: "".into(),
            total_pages: 0,
            error: Some("File không tồn tại!".into()),
        });
    }

    match extract_text_from_pdf(&path) {
        Ok(pages) => {
            let total_pages = pages.len();
            let first_page_text = pages.first().map(|(_, t)| t.as_str()).unwrap_or("");
            let (loai, so, ngay, co_quan, trich_yeu) = catalog::extract_document_info(first_page_text);
            Json(ExtractCatalogRes {
                success: true,
                loai_van_ban: loai,
                so_ky_hieu: so,
                ngay_ban_hanh: ngay,
                co_quan_ban_hanh: co_quan,
                trich_yeu,
                total_pages,
                error: None,
            })
        }
        Err(e) => Json(ExtractCatalogRes {
            success: false,
            loai_van_ban: "".into(),
            so_ky_hieu: "".into(),
            ngay_ban_hanh: "".into(),
            co_quan_ban_hanh: "".into(),
            trich_yeu: "".into(),
            total_pages: 0,
            error: Some(e),
        }),
    }
}

#[derive(Deserialize)]
struct ExportCatalogReq {
    rows: Vec<catalog::CatalogRow>,
    title: String,
}

async fn export_catalog_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExportCatalogReq>,
) -> Response {
    let temp_docx = state.data_dir.join("temp").join("catalog_export.docx");
    let _ = catalog::export_catalog_to_docx(&temp_docx, &payload.rows, &payload.title);

    match fs::read(&temp_docx) {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"Danh_Muc_Ho_So_Mau03.docx\""),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Lỗi tạo file Word").into_response(),
    }
}

async fn get_voices_handler() -> Json<Vec<voice::VoiceOption>> {
    Json(voice::get_available_voices())
}

#[derive(Deserialize)]
struct VoiceSynthesizeReq {
    text: String,
    voice: String,
    rate: i32,
}

#[derive(Serialize)]
struct VoiceSynthesizeRes {
    audio_url: String,
    srt_url: String,
    error: Option<String>,
}

async fn synthesize_voice_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VoiceSynthesizeReq>,
) -> Json<VoiceSynthesizeRes> {
    let rand_id = uuid::Uuid::new_v4().simple().to_string();
    let mp3_name = format!("voice_{}.mp3", rand_id);
    let srt_name = format!("sub_{}.srt", rand_id);

    let mp3_path = state.data_dir.join("temp").join(&mp3_name);
    let srt_path = state.data_dir.join("temp").join(&srt_name);

    match voice::synthesize_speech(&payload.text, &payload.voice, payload.rate, &mp3_path).await {
        Ok(bytes) => {
            let duration = (bytes.len() as f64 / 6000.0).max(1.0);
            let _ = voice::generate_srt(&payload.text, duration, &srt_path);

            Json(VoiceSynthesizeRes {
                audio_url: format!("/api/voice/file/{}", mp3_name),
                srt_url: format!("/api/voice/file/{}", srt_name),
                error: None,
            })
        }
        Err(e) => Json(VoiceSynthesizeRes {
            audio_url: "".into(),
            srt_url: "".into(),
            error: Some(e),
        }),
    }
}

async fn serve_temp_file_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Response {
    let file_path = state.data_dir.join("temp").join(&name);
    if let Ok(bytes) = fs::read(file_path) {
        let content_type = if name.ends_with(".mp3") {
            "audio/mpeg"
        } else if name.ends_with(".srt") {
            "text/plain; charset=utf-8"
        } else {
            "application/octet-stream"
        };
        ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Không tìm thấy file").into_response()
    }
}

async fn get_officers_handler(State(state): State<Arc<AppState>>) -> Json<Vec<kpi::Officer>> {
    let conn = state.db.lock().unwrap();
    let officers = kpi::get_officers(&conn).unwrap_or_default();
    Json(officers)
}

async fn get_tasks_handler(State(state): State<Arc<AppState>>) -> Json<Vec<kpi::TaskCatalogItem>> {
    let conn = state.db.lock().unwrap();
    let tasks = kpi::get_tasks(&conn).unwrap_or_default();
    Json(tasks)
}

#[derive(Deserialize)]
struct KpiFilterReq {
    thang: Option<i32>,
    nam: Option<i32>,
}

async fn get_assignments_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<KpiFilterReq>,
) -> Json<Vec<kpi::Assignment>> {
    let thang = params.thang.unwrap_or(6);
    let nam = params.nam.unwrap_or(2026);
    let conn = state.db.lock().unwrap();
    let list = kpi::get_assignments(&conn, thang, nam).unwrap_or_default();
    Json(list)
}

#[derive(Deserialize)]
struct AddAssignmentReq {
    officer_id: i64,
    task_id: i64,
    thang: i32,
    quy: i32,
    nam: i32,
    chat_luong: String,
    tien_do: String,
    ghi_chu: Option<String>,
}

async fn add_assignment_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddAssignmentReq>,
) -> Json<bool> {
    let conn = state.db.lock().unwrap();
    let diem_goc: f64 = conn
        .query_row("SELECT diem_goc FROM task_catalog WHERE id = ?1", rusqlite::params![payload.task_id], |r| r.get(0))
        .unwrap_or(10.0);

    let diem_dat = kpi::calculate_score(diem_goc, &payload.chat_luong, &payload.tien_do);

    let res = conn.execute(
        "INSERT INTO assignments (officer_id, task_id, thang, quy, nam, chat_luong, tien_do, diem_dat, ghi_chu) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            payload.officer_id,
            payload.task_id,
            payload.thang,
            payload.quy,
            payload.nam,
            payload.chat_luong,
            payload.tien_do,
            diem_dat,
            payload.ghi_chu
        ],
    );

    Json(res.is_ok())
}

async fn get_kpi_summary_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<KpiFilterReq>,
) -> Json<Vec<kpi::OfficerKpiSummary>> {
    let thang = params.thang.unwrap_or(6);
    let nam = params.nam.unwrap_or(2026);
    let conn = state.db.lock().unwrap();
    let list = kpi::get_kpi_summary(&conn, thang, nam).unwrap_or_default();
    Json(list)
}

// ==========================================
// API SCAN OCR VĂN BẢN
// ==========================================

async fn ocr_status_handler() -> impl IntoResponse {
    Json(ocr::check_ocr_status())
}

async fn ocr_scan_batch_handler(mut multipart: Multipart) -> impl IntoResponse {
    let mut files: Vec<(String, usize, Vec<u8>)> = Vec::new();
    let mut lang = "vie".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "lang" {
            if let Ok(text) = field.text().await {
                lang = text;
            }
        } else {
            let filename = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("page_{}.png", files.len() + 1));

            // Extract custom sequence if sent as field name e.g. "file_0", "file_1"
            let order = if let Some(stripped) = name.strip_prefix("file_") {
                stripped.parse::<usize>().unwrap_or(files.len())
            } else {
                files.len()
            };

            if let Ok(bytes) = field.bytes().await {
                if !bytes.is_empty() {
                    files.push((filename, order, bytes.to_vec()));
                }
            }
        }
    }

    // Sort files by explicit order specified by client
    files.sort_by_key(|f| f.1);

    let mut pages = Vec::new();
    let mut total_words = 0;
    let mut total_chars = 0;
    let mut combined_lines = Vec::new();

    for (idx, (filename, _, bytes)) in files.into_iter().enumerate() {
        let page_num = idx + 1;
        let ocr_text = ocr::ocr_single_image(&bytes, &lang)
            .unwrap_or_else(|e| format!("[Lỗi nhận dạng: {}]", e));
        let word_count = ocr_text.split_whitespace().count();
        let char_count = ocr_text.chars().count();

        total_words += word_count;
        total_chars += char_count;

        combined_lines.push(format!("--- TRANG {} ({}) ---\n{}\n", page_num, filename, ocr_text));

        pages.push(ocr::PageOcrResult {
            page_number: page_num,
            filename,
            text: ocr_text,
            word_count,
            char_count,
        });
    }

    let combined_text = combined_lines.join("\n");
    let total_pages = pages.len();

    Json(ocr::BatchOcrResponse {
        success: true,
        total_pages,
        total_words,
        total_chars,
        pages,
        combined_text,
    })
}

async fn ocr_export_docx_handler(
    Json(payload): Json<ocr::ExportDocxRequest>,
) -> impl IntoResponse {
    match ocr::generate_docx(&payload.title, &payload.pages) {
        Ok(bytes) => (
            [
                (
                    header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"TaiLieu_Scan_OCR.docx\"",
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi xuất DOCX: {}", err),
        )
            .into_response(),
    }
}

// ==========================================
// API BIỂU MẪU VĂN BẢN HÀNH CHÍNH (NĐ 30/2020)
// ==========================================

async fn get_admin_doc_templates_handler() -> impl IntoResponse {
    Json(admin_doc::get_templates())
}

async fn export_admin_docx_handler(
    Json(payload): Json<admin_doc::ExportAdminDocRequest>,
) -> impl IntoResponse {
    match admin_doc::generate_admin_docx(&payload.template_id, &payload.fields) {
        Ok(bytes) => {
            let filename = format!("VanBan_{}.docx", payload.template_id);
            let disposition = format!("attachment; filename=\"{}\"", filename);
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                ),
            );
            if let Ok(val) = axum::http::HeaderValue::from_str(&disposition) {
                headers.insert(header::CONTENT_DISPOSITION, val);
            }
            (headers, bytes).into_response()
        }
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi xuất văn bản hành chính: {}", err),
        )
            .into_response(),
    }
}

// ==========================================
// API QUẢN LÝ TIẾN TRÌNH & GIẢI PHÓNG PHẦN CỨNG
// ==========================================

async fn heartbeat_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let now = chrono::Utc::now().timestamp();
    state.last_heartbeat.store(now, Ordering::SeqCst);
    Json(serde_json::json!({ "status": "ok", "timestamp": now }))
}

async fn shutdown_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    println!("  -> Nhận yêu cầu thoát ứng dụng. Đang dừng các tác vụ nền...");
    state.stop_flag.store(true, Ordering::SeqCst);
    if let Ok(mut status) = state.indexing_status.lock() {
        status.stopped = true;
        status.is_running = false;
        status.message = "Đã dừng tiến trình để thoát ứng dụng".into();
    }

    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        println!("  -> Đã giải phóng toàn bộ RAM, CPU và đóng tiến trình thành công.");
        std::process::exit(0);
    });

    Json(serde_json::json!({
        "status": "shutting_down",
        "message": "Đang tắt ứng dụng và giải phóng toàn bộ tài nguyên"
    }))
}



