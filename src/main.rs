mod catalog;
mod db;
mod kpi;
mod search;
mod state;
mod ui;
mod utils;
mod voice;

use axum::{
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Local;
use search::{extract_text_from_docx, extract_text_from_pdf, search_documents, SearchResultItem};
use serde::{Deserialize, Serialize};
use state::AppState;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utils::remove_accents;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Khởi tạo thư mục dữ liệu
    let data_dir = PathBuf::from("cya_data");
    fs::create_dir_all(&data_dir)?;
    let temp_dir = data_dir.join("temp");
    fs::create_dir_all(&temp_dir)?;

    // Khởi tạo SQLite
    let db_path = data_dir.join("cya_suite.db");
    let conn = db::init_db(&db_path)?;
    let shared_state = Arc::new(AppState::new(conn, data_dir));

    let app = Router::new()
        .route("/", get(index_handler))
        // API Tìm kiếm
        .route("/api/index", post(index_folder_handler))
        .route("/api/search", get(search_handler))
        .route("/api/open-file", get(open_file_handler))
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
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    let port = 8080;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://127.0.0.1:{}", port);

    println!("============================================================");
    println!("  CYA SUITE - HỆ THỐNG VĂN PHÒNG & AI HIỆU NĂNG CAO (RUST)");
    println!("  Tác giả: Nguyễn Tiến Hùng (cyaNguyen)");
    println!("  Facebook: https://www.facebook.com/nguyen.tien.hung.409785/");
    println!("  Telegram: @cyanguyen | GitHub: https://github.com/cyaNguyen");
    println!("  Bản quyền: VĨNH VIỄN (Không giới hạn thời gian)");
    println!("============================================================");
    println!("  -> Máy chủ đang chạy tại: {}", url);
    println!("  -> Đang tự động mở trình duyệt...");

    // Tự động mở trình duyệt
    let _ = webbrowser::open(&url);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(ui::INDEX_HTML)
}

#[derive(Deserialize)]
struct IndexFolderReq {
    folder: String,
}

#[derive(Serialize)]
struct IndexFolderRes {
    success: bool,
    indexed_files: usize,
    message: String,
}

async fn index_folder_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IndexFolderReq>,
) -> Json<IndexFolderRes> {
    let folder = payload.folder.trim().to_string();
    let folder_path = PathBuf::from(&folder);

    if !folder_path.exists() || !folder_path.is_dir() {
        return Json(IndexFolderRes {
            success: false,
            indexed_files: 0,
            message: "Thư mục không tồn tại hoặc không hợp lệ!".into(),
        });
    }

    let db_arc = state.db.clone();
    let mut count = 0;
    let now_str = Local::now().format("%d/%m/%Y %H:%M:%S").to_string();

    // Thu thập file PDF & DOCX
    let mut files = Vec::new();
    for entry in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "pdf" || ext_lower == "docx" {
                    files.push((p.to_path_buf(), ext_lower));
                }
            }
        }
    }

    // Quét và nạp dữ liệu
    for (file_p, ext) in files {
        let file_name = file_p.file_name().unwrap_or_default().to_string_lossy().to_string();
        let file_path_str = file_p.to_string_lossy().to_string();

        if ext == "docx" {
            if let Ok(text) = extract_text_from_docx(&file_p) {
                if !text.trim().is_empty() {
                    let text_no_acc = remove_accents(&text);
                    let conn = db_arc.lock().unwrap();
                    let _ = conn.execute(
                        "INSERT INTO documents (file_path, file_name, file_type, page_number, content, content_no_accent, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![file_path_str, file_name, "WORD", 1, text, text_no_acc, now_str],
                    );
                    let _ = conn.execute(
                        "INSERT INTO documents_fts (file_path, file_name, file_type, page_number, content, content_no_accent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![file_path_str, file_name, "WORD", 1, text, text_no_acc],
                    );
                    count += 1;
                }
            }
        } else if ext == "pdf" {
            if let Ok(pages) = extract_text_from_pdf(&file_p) {
                let conn = db_arc.lock().unwrap();
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
                count += 1;
            }
        }
    }

    Json(IndexFolderRes {
        success: true,
        indexed_files: count,
        message: format!("Đã quét và lập chỉ mục thành công {} file tài liệu!", count),
    })
}

#[derive(Deserialize)]
struct SearchReq {
    q: Option<String>,
}

async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchReq>,
) -> Json<Vec<SearchResultItem>> {
    let q = params.q.unwrap_or_default();
    let conn = state.db.lock().unwrap();
    let results = search_documents(&conn, &q).unwrap_or_default();
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
