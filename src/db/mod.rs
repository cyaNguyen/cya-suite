use rusqlite::{Connection, Result};
use std::path::Path;

pub fn init_db<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // Bật chế độ WAL và tối ưu hiệu năng SQLite
    let _: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |r| r.get(0))?;
    conn.execute_batch(
        "
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA temp_store = MEMORY;
        PRAGMA cache_size = -64000;
        ",
    )?;

    // Tạo bảng tìm kiếm văn bản (Search & FTS5)
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_type TEXT NOT NULL,
            page_number INTEGER NOT NULL,
            content TEXT NOT NULL,
            content_no_accent TEXT NOT NULL,
            indexed_at TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            file_path,
            file_name,
            file_type,
            page_number UNINDEXED,
            content,
            content_no_accent,
            tokenize='unicode61'
        );

        CREATE INDEX IF NOT EXISTS idx_docs_path ON documents(file_path);
        CREATE INDEX IF NOT EXISTS idx_docs_name ON documents(file_name);
        ",
    )?;

    // Tạo bảng Lập danh mục hồ sơ
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS catalog_rows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            stt INTEGER,
            so_ky_hieu TEXT,
            ngay_ban_hanh TEXT,
            co_quan_ban_hanh TEXT,
            trich_yeu TEXT,
            to_so TEXT,
            do_mat TEXT DEFAULT 'Thường',
            ghi_chu TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )?;

    // Tạo bảng KPI PX05 (Quản lý & Đánh giá cán bộ)
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS officers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ho_ten TEXT NOT NULL,
            cap_bac TEXT,
            chuc_vu TEXT NOT NULL,
            linh_vuc_phu_trach TEXT,
            active INTEGER DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS task_catalog (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nhom TEXT NOT NULL,
            ten_nhiem_vu TEXT NOT NULL,
            san_pham TEXT,
            diem_goc REAL NOT NULL,
            active INTEGER DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS assignments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            officer_id INTEGER NOT NULL,
            task_id INTEGER NOT NULL,
            thang INTEGER NOT NULL,
            quy INTEGER NOT NULL,
            nam INTEGER NOT NULL,
            han_hoan_thanh TEXT,
            noi_dung_cu_the TEXT,
            trang_thai TEXT DEFAULT 'hoan_thanh',
            chat_luong TEXT DEFAULT 'dat_yeu_cau',
            tien_do TEXT DEFAULT 'dung_han',
            diem_dat REAL DEFAULT 0,
            ghi_chu TEXT,
            ngay_hoan_thanh TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(officer_id) REFERENCES officers(id),
            FOREIGN KEY(task_id) REFERENCES task_catalog(id)
        );

        CREATE TABLE IF NOT EXISTS period_locks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            loai TEXT NOT NULL, -- 'month' hoặc 'quarter'
            thang INTEGER,
            quy INTEGER,
            nam INTEGER NOT NULL,
            locked INTEGER DEFAULT 0,
            nguoi_khoa TEXT,
            thoi_gian_khoa TEXT,
            UNIQUE(loai, thang, quy, nam)
        );
        ",
    )?;

    // Seed cán bộ và nhiệm vụ mẫu nếu bảng trống
    let officer_count: i64 = conn.query_row("SELECT COUNT(*) FROM officers", [], |r| r.get(0))?;
    if officer_count == 0 {
        conn.execute(
            "INSERT INTO officers (ho_ten, cap_bac, chuc_vu, linh_vuc_phu_trach) VALUES 
             ('Nguyễn Văn An', 'Trung tá', 'Chỉ huy / Quản trị hệ thống', 'Quản trị điều hành chung'),
             ('Lê Văn Nam', 'Đại úy', 'Cán bộ thanh tra', 'Thanh tra hành chính và giải quyết khiếu nại'),
             ('Trần Thị Mai', 'Thượng úy', 'Cán bộ tổng hợp', 'Công tác hồ sơ, tài liệu và văn thư')",
            [],
        )?;
    }

    let task_count: i64 = conn.query_row("SELECT COUNT(*) FROM task_catalog", [], |r| r.get(0))?;
    if task_count == 0 {
        conn.execute(
            "INSERT INTO task_catalog (nhom, ten_nhiem_vu, san_pham, diem_goc) VALUES 
             ('Công tác chuyên môn', 'Tiến hành cuộc thanh tra theo kế hoạch', 'Kết luận thanh tra', 30.0),
             ('Công tác chuyên môn', 'Xác minh đơn thư khiếu nại, tố cáo', 'Báo cáo xác minh', 25.0),
             ('Công tác hành chính', 'Lập báo cáo định kỳ tháng/quý', 'Văn bản báo cáo', 15.0),
             ('Công tác hồ sơ', 'Thu thập, chỉnh lý và lập danh mục hồ sơ lưu trữ', 'Danh mục hồ sơ chuẩn', 20.0),
             ('Nhiệm vụ đột xuất', 'Thực hiện nhiệm vụ đột xuất do lãnh đạo phân công', 'Sản phẩm hoàn thành', 10.0)",
            [],
        )?;
    }

    Ok(conn)
}
