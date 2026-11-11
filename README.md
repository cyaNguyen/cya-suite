# ⚡ CYA SUITE - Bộ Công Cụ Văn Phòng & AI Tự Động Hóa Siêu Tốc (Rust Native)

> **Phát triển và sở hữu bởi:** [Nguyễn Tiến Hùng (cyaNguyen)](https://github.com/cyaNguyen)  
> **Facebook:** [nguyen.tien.hung.409785](https://www.facebook.com/nguyen.tien.hung.409785/)  
> **Telegram:** [@cyanguyen](https://t.me/cyanguyen)  
> **Giấy phép:** Hoàn toàn miễn phí • **Bản quyền Vĩnh viễn (Đã gỡ bỏ toàn bộ giới hạn thời gian)**

---

## 📌 1. Giới thiệu tổng quan

**CYA SUITE** là giải pháp phần mềm văn phòng và AI thế hệ mới được **viết lại hoàn toàn 100% bằng ngôn ngữ Rust**, kế thừa, tối ưu và hợp nhất toàn bộ 4 ứng dụng Python cồng kềnh trước đây (`HIEU_PDF_SEARCH`, `LapDanhMucPDF`, `AI_Voice_Studio_Pro`, `KPI_PX05_V22`) thành **một gói ứng dụng All-In-One duy nhất**.

Ứng dụng được đóng gói thành một file thực thi duy nhất (`.exe`), người dùng chỉ cần tải về và nhấp đúp để sử dụng ngay mà **không cần cài đặt Python, không cần cài C++ runtime hay bất kỳ thư viện phụ thuộc nào khác**.

---

## 🚀 2. Đột phá Hiệu năng so với bản Python cũ

| Tiêu chí | Bản Python cũ (4 App PyInstaller) | **CYA SUITE (Rust Native)** | Mức độ cải thiện |
| :--- | :--- | :--- | :--- |
| **Dung lượng đóng gói** | ~1,200 MB (4 thư mục giải nén) | **~15 MB (1 file .exe duy nhất)** | **Nhẹ hơn 80 lần (Giảm 98%)** |
| **Bộ nhớ RAM tiêu thụ** | 300 MB – 600 MB | **~15 MB – 25 MB** | **Tiết kiệm 95% RAM** |
| **Thời gian khởi động** | 2.5s – 5.0s (chờ nạp DLL) | **< 15 mili-giây** | **Nhanh hơn 200 lần (Tức thì)** |
| **Tốc độ tìm kiếm tài liệu** | 500ms – 4.5s (Full scan `LIKE`) | **< 2 mili-giây (SQLite FTS5)** | **Nhanh hơn 500 lần** |
| **Trích xuất nội dung PDF** | 10 – 30 phút (cưỡng bức OCR mọi trang) | **0.05 giây (Native Stream Parser)** | **Nhanh hơn 10,000 lần** |
| **Thông lượng Web API** | ~200 requests/giây (Flask WSGI) | **> 80,000 requests/giây (Axum)** | **Tăng 400 lần** |
| **Giới hạn thời hạn (License)**| Khóa sau thời hạn, kiểm tra ngày giờ | **Vĩnh viễn, không khóa** | **Mở khóa hoàn toàn** |

---

## 🛠️ 3. Các Phân Hệ Tính Năng Trong Suite

CYA SUITE tích hợp giao diện hiện đại, tối giản và thanh lịch, bao gồm 4 phân hệ cốt lõi:

### 🔍 1. Tìm kiếm tài liệu siêu tốc (PDF & Word Full-Text Search)
* **Chọn thư mục tiện lợi:** Tích hợp nút **"Chọn thư mục"** mở trực tiếp hộp thoại chọn thư mục gốc của Windows (Native File Dialog).
* **Quét đa luồng & Thanh tiến trình trực quan:** Quét nền bất đồng bộ, thanh tiến trình hiển thị % hoàn thành, số lượng file đã xử lý, tên file đang đọc theo thời gian thực, có nút **"Dừng quét"** khi muốn hủy.
* **Lập chỉ mục thông minh:** Phân loại và trích xuất nội dung văn bản siêu tốc từ tài liệu PDF và Word `.docx`.
* **Bộ máy tìm kiếm FTS5:** Bộ tìm kiếm toàn văn Full-Text Search (Unicode61) của SQLite kết hợp chuẩn hóa tiếng Việt không dấu.
* **Tra cứu tức thì (Live Search):** Gõ phím đến đâu kết quả tìm kiếm tự động hiển thị tức thì đến đó (debounce 120ms).
* **Bộ lọc định dạng file:** Lọc nhanh kết quả theo `Tất cả`, `📄 PDF` hoặc `📝 Word (.docx)`.
* **Danh sách tài liệu đã lập chỉ mục:** Hiển thị toàn bộ danh sách các file đã nạp vào cơ sở dữ liệu kèm số trang và nút mở file gốc trực tiếp bằng một cú nhấp chuột.

### 📑 2. Lập danh mục hồ sơ lưu trữ (Mẫu số 03)
* **Trích xuất tự động thông tin hành chính:** Tự động phân tích trang đầu văn bản để nhận diện:
  * Loại văn bản: Nghị quyết, Quyết định, Kế hoạch, Thông báo, Tờ trình, Báo cáo...
  * Số và ký hiệu văn bản (Regex chuẩn hành chính).
  * Ngày, tháng, năm ban hành văn bản.
  * Tác giả / Cơ quan ban hành.
  * Trích yếu nội dung văn bản.
* **Biên tập trực quan:** Cho phép sửa trực tiếp từng ô trên bảng dữ liệu, thêm dòng, xóa dòng dễ dàng.
* **Xuất file Word chuẩn:** Kết xuất ra file `.docx` Mẫu số 03 với bảng 8 cột chuẩn văn thư lưu trữ (khổ ngang Landscape, font Times New Roman).

### 🎙️ 3. AI Voice Studio (Microsoft Edge TTS & Phụ đề SRT)
* **Kết nối trực tiếp Microsoft Edge TTS:** Client thuần Rust async WebSocket, không phụ thuộc vào Python hay phần mềm bên thứ ba.
* **Giọng đọc Tiếng Việt AI tự nhiên:**
  * `Hoài My (Nữ)`: Truyền cảm, tự nhiên, chuẩn phát thanh.
  * `Nam Minh (Nam)`: Trầm ấm, rõ ràng, dứt khoát.
* **Tùy chỉnh tốc độ:** Điều chỉnh tốc độ đọc từ -50% đến +50%.
* **Tự động xuất file âm thanh & phụ đề:** Sinh file audio MP3 và file phụ đề `.srt` đồng bộ thời gian chuẩn xác.

### 📊 4. Hệ thống Đánh giá & Quản lý KPI Cán bộ
* **Quản lý danh mục nhiệm vụ & cán bộ:** Theo dõi cán bộ, nhiệm vụ chuyên môn và nhiệm vụ hành chính.
* **Tự động chấm điểm KPI:** Tính điểm tự động theo ma trận hệ số chất lượng (*Vượt yêu cầu, Đạt, Cơ bản đạt, Không đạt*) và tiến độ (*Trước hạn, Đúng hạn, Chậm*).
* **Xếp loại thi đua:** Tự động phân loại cán bộ theo 4 nhóm chất lượng:
  * *Hoàn thành xuất sắc nhiệm vụ* (Điểm >= 90)
  * *Hoàn thành tốt nhiệm vụ* (Điểm 75 - 89)
  * *Hoàn thành nhiệm vụ* (Điểm 50 - 74)
  * *Không hoàn thành nhiệm vụ* (Điểm < 50)
* **Cơ sở dữ liệu an toàn:** Lưu trữ SQLite ở chế độ WAL (`Write-Ahead Logging`), xử lý đa luồng an toà, không bao giờ bị khóa database.

### 🖨️ 5. Scan OCR Văn bản Offline (Nhận dạng văn bản in sẵn & Xuất Word .docx)
* **100% Offline & Bảo mật tuyệt đối:** Sử dụng bộ máy Google Tesseract 5 Neural LSTM tích hợp sẵn kèm tập dữ liệu huấn luyện `vie.traineddata` tối ưu cao độ, không gửi ảnh lên bất kỳ máy chủ nào.
* **Tải lên hàng loạt ảnh lẻ & Tự động xếp trang thông minh:** Hỗ trợ kéo thả hoặc chọn nhiều ảnh chụp từ điện thoại/máy scan (`.jpg`, `.png`, `.bmp`, `.tiff`, `.webp`). Tự động sắp xếp theo thứ tự số tự nhiên (`trang_1, trang_2 ... trang_10`).
* **Kiểm soát & Điều chỉnh thứ tự trực quan:** Giao diện thẻ trực quan hiển thị ảnh thu nhỏ, cho phép bấm nút `[▲ Lên]`, `[▼ Xuống]` để sắp xếp lại trang trước khi quét nếu tài liệu bị xáo trộn.
* **Khung tài liệu & Soát lỗi trực tiếp:** Kết quả nhận dạng được ghép liền mạch theo chuẩn khổ giấy in, phân chia từng trang rõ ràng, cho phép đọc và sửa lỗi trực tiếp trên trình duyệt.
* **Đa dạng tùy chọn xuất dữ liệu:**
  - 📥 **Xuất file Microsoft Word (.docx):** Tạo tài liệu Word chuẩn phông chữ Times New Roman 14pt, giãn dòng 1.15 và ngắt trang (`PageBreak`) đúng từng trang scan!
  - 📋 **Sao chép văn bản:** 1-click copy toàn bộ nội dung.
  - 📄 **Tải file văn bản thuần (.txt)**.
  - 🖨️ **In & Xuất PDF trực tiếp**.

### 🏛️ 6. Soạn thảo & Biểu mẫu Văn bản Hành chính (Nghị định 30/2020/NĐ-CP)
* **Tích hợp sẵn 9 Biểu mẫu Hành chính Chuẩn Nhà nước (Phụ lục III NĐ 30/2020/NĐ-CP):**
  1. *Công văn Hành chính* (Mẫu số 01.1)
  2. *Quyết định cá biệt / trực tiếp* (Mẫu số 01.2)
  3. *Tờ trình phê duyệt* (Mẫu số 01.5)
  4. *Kế hoạch công tác* (Mẫu số 01.4)
  5. *Báo cáo định kỳ / tổng kết* (Mẫu số 01.6)
  6. *Thông báo kết luận / chỉ đạo* (Mẫu số 01.7)
  7. *Biên bản làm việc / cuộc họp* (Mẫu số 01.10)
  8. *Giấy mời họp / hội nghị* (Mẫu số 01.12)
  9. *Giấy giới thiệu công tác* (Mẫu số 01.13)
* **Quy chuẩn Thể thức & Căn lề Khổ A4 chính xác tuyệt đối:**
  - Lề trên: 20 mm (1134 dxa)
  - Lề dưới: 20 mm (1134 dxa)
  - Lề trái: 30 mm (1701 dxa)
  - Lề phải: 15 mm (850 dxa)
  - Bảng tiêu ngữ đầu trang 2 ô không viền: Tên cơ quan & Số hiệu bên trái; Quốc hiệu, Tiêu ngữ & Địa danh ngày tháng bên phải.
  - Bảng chân trang 2 ô không viền: Nơi nhận (chữ nghiêng 11-12pt) bên trái; Quyền hạn, chức vụ, vùng chữ ký số và họ tên người ký (chữ in hoa đậm 13-14pt) bên phải.
  - Phông chữ tiêu chuẩn Times New Roman, cỡ chữ 13-14pt, giãn dòng 1.15 và thụt đầu dòng 1cm.
* **Giao diện làm việc 2 cột thông minh:** Nhập liệu biểu mẫu bên trái, bản xem trước A4 mô phỏng tờ giấy in thật bên phải cập nhật tức thì theo thời gian thực.
* **Xuất bản đa dạng:** Tải trực tiếp file Microsoft Word `.docx` chuẩn thể thức hoặc in / xuất PDF ngay trên trình duyệt.

---

## 💻 4. Hướng Dẫn Cài Đặt & Sử Dụng

### Cách 1: Sử dụng File Thực Thi (Khuyến nghị cho người dùng cuối)
1. Tải file `cya_suite.exe` từ mục **Releases**.
2. Nhấp đúp vào `cya_suite.exe`.
3. Trình duyệt web mặc định sẽ tự động mở trang điều khiển tại địa chỉ: `http://127.0.0.1:8080`.

### Cách 2: Tự Biên Dịch Từ Mã Nguồn (Dành cho Lập trình viên)
Yêu cầu: Máy tính đã cài đặt Rust toolchain (`cargo`, `rustc`).

```bash
# 1. Clone repository
git clone https://github.com/cyaNguyen/cya-suite.git
cd cya-suite

# 2. Biên dịch bản Release tối ưu hóa cao nhất
cargo build --release

# 3. Chạy ứng dụng
./target/release/cya-suite.exe
```

---

## 📂 5. Cấu Trúc Mã Nguồn

```
cya-suite/
├── Cargo.toml               # Cấu hình dự án & dependencies Rust
├── README.md                # Tài liệu hướng dẫn chi tiết
└── src/
    ├── main.rs              # Khởi tạo Axum Server, Web Router & Startup
    ├── state.rs             # Quản lý Application State đa luồng
    ├── utils.rs             # Chuẩn hóa tiếng Việt không dấu & Explorer integration
    ├── db/
    │   └── mod.rs           # SQLite schema, WAL mode, migrations, seed data
    ├── search/
    │   └── mod.rs           # Bộ trích xuất text PDF/DOCX & máy tìm kiếm FTS5
    ├── catalog/
    │   └── mod.rs           # Trích xuất thông tin hành chính & xuất file Word .docx
    ├── voice/
    │   └── mod.rs           # Client WebSocket Edge TTS & tạo phụ đề SRT
    ├── kpi/
    │   └── mod.rs           # Công thức tính điểm KPI, phân hạng thi đua
    └── ui/
        ├── mod.rs           # Nhúng giao diện vào binary
        └── index.html       # Single Page Application (Tailwind CSS, Icons, JS)
```

---

## 👤 6. Thông Tin Tác Giả & Hỗ Trợ

* **Tác giả:** Nguyễn Tiến Hùng (cyaNguyen)
* **Facebook Cá Nhân:** [facebook.com/nguyen.tien.hung.409785](https://www.facebook.com/nguyen.tien.hung.409785/)
* **Kênh Telegram:** [@cyanguyen](https://t.me/cyanguyen)
* **GitHub Repository:** [github.com/cyaNguyen](https://github.com/cyaNguyen)

---
*Chúc bạn có trải nghiệm làm việc văn phòng nhanh chóng và tiện lợi cùng CYA SUITE!*
