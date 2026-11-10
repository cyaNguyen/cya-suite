use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Officer {
    pub id: i64,
    pub ho_ten: String,
    pub cap_bac: Option<String>,
    pub chuc_vu: String,
    pub linh_vuc_phu_trach: Option<String>,
    pub active: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCatalogItem {
    pub id: i64,
    pub nhom: String,
    pub ten_nhiem_vu: String,
    pub san_pham: Option<String>,
    pub diem_goc: f64,
    pub active: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub id: i64,
    pub officer_id: i64,
    pub officer_name: Option<String>,
    pub task_id: i64,
    pub task_name: Option<String>,
    pub thang: i32,
    pub quy: i32,
    pub nam: i32,
    pub han_hoan_thanh: Option<String>,
    pub noi_dung_cu_the: Option<String>,
    pub trang_thai: String,
    pub chat_luong: String,
    pub tien_do: String,
    pub diem_dat: f64,
    pub ghi_chu: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficerKpiSummary {
    pub officer_id: i64,
    pub officer_name: String,
    pub chuc_vu: String,
    pub total_tasks: i32,
    pub total_score: f64,
    pub xep_loai: String,
}

/// Tính điểm KPI dựa trên điểm gốc, chất lượng và tiến độ
pub fn calculate_score(diem_goc: f64, chat_luong: &str, tien_do: &str) -> f64 {
    let q_mult = match chat_luong {
        "vuot_yeu_cau" => 1.2,
        "dat_yeu_cau" => 1.0,
        "co_ban_dat" => 0.8,
        "khong_dat" => 0.5,
        _ => 1.0,
    };
    let p_mult = match tien_do {
        "truoc_han" => 1.1,
        "dung_han" => 1.0,
        "cham_chap_thuan" => 0.9,
        "cham_han" => 0.7,
        _ => 1.0,
    };
    (diem_goc * q_mult * p_mult * 10.0).round() / 10.0
}

/// Xếp loại thi đua dựa trên tổng điểm
pub fn classify_score(score: f64) -> &'static str {
    if score >= 90.0 {
        "Hoàn thành xuất sắc nhiệm vụ"
    } else if score >= 75.0 {
        "Hoàn thành tốt nhiệm vụ"
    } else if score >= 50.0 {
        "Hoàn thành nhiệm vụ"
    } else {
        "Không hoàn thành nhiệm vụ"
    }
}

pub fn get_officers(conn: &Connection) -> Result<Vec<Officer>> {
    let mut stmt = conn.prepare(
        "SELECT id, ho_ten, cap_bac, chuc_vu, linh_vuc_phu_trach, active FROM officers ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Officer {
            id: row.get(0)?,
            ho_ten: row.get(1)?,
            cap_bac: row.get(2)?,
            chuc_vu: row.get(3)?,
            linh_vuc_phu_trach: row.get(4)?,
            active: row.get(5)?,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn get_tasks(conn: &Connection) -> Result<Vec<TaskCatalogItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, nhom, ten_nhiem_vu, san_pham, diem_goc, active FROM task_catalog ORDER BY nhom, id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TaskCatalogItem {
            id: row.get(0)?,
            nhom: row.get(1)?,
            ten_nhiem_vu: row.get(2)?,
            san_pham: row.get(3)?,
            diem_goc: row.get(4)?,
            active: row.get(5)?,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn get_assignments(conn: &Connection, thang: i32, nam: i32) -> Result<Vec<Assignment>> {
    let mut stmt = conn.prepare(
        "
        SELECT a.id, a.officer_id, o.ho_ten, a.task_id, t.ten_nhiem_vu,
               a.thang, a.quy, a.nam, a.han_hoan_thanh, a.noi_dung_cu_the,
               a.trang_thai, a.chat_luong, a.tien_do, a.diem_dat, a.ghi_chu
        FROM assignments a
        JOIN officers o ON o.id = a.officer_id
        JOIN task_catalog t ON t.id = a.task_id
        WHERE a.thang = ?1 AND a.nam = ?2
        ORDER BY a.id DESC
        ",
    )?;
    let rows = stmt.query_map(params![thang, nam], |row| {
        Ok(Assignment {
            id: row.get(0)?,
            officer_id: row.get(1)?,
            officer_name: Some(row.get(2)?),
            task_id: row.get(3)?,
            task_name: Some(row.get(4)?),
            thang: row.get(5)?,
            quy: row.get(6)?,
            nam: row.get(7)?,
            han_hoan_thanh: row.get(8)?,
            noi_dung_cu_the: row.get(9)?,
            trang_thai: row.get(10)?,
            chat_luong: row.get(11)?,
            tien_do: row.get(12)?,
            diem_dat: row.get(13)?,
            ghi_chu: row.get(14)?,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn get_kpi_summary(conn: &Connection, thang: i32, nam: i32) -> Result<Vec<OfficerKpiSummary>> {
    let mut stmt = conn.prepare(
        "
        SELECT o.id, o.ho_ten, o.chuc_vu,
               COUNT(a.id) as total_tasks,
               COALESCE(SUM(a.diem_dat), 0.0) as total_score
        FROM officers o
        LEFT JOIN assignments a ON a.officer_id = o.id AND a.thang = ?1 AND a.nam = ?2
        WHERE o.active = 1
        GROUP BY o.id, o.ho_ten, o.chuc_vu
        ORDER BY total_score DESC
        ",
    )?;
    let rows = stmt.query_map(params![thang, nam], |row| {
        let score: f64 = row.get(4)?;
        let xep_loai = classify_score(score).to_string();
        Ok(OfficerKpiSummary {
            officer_id: row.get(0)?,
            officer_name: row.get(1)?,
            chuc_vu: row.get(2)?,
            total_tasks: row.get(3)?,
            total_score: score,
            xep_loai,
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}
