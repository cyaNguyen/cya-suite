use docx_rs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDocTemplate {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub decree_ref: String,
    pub default_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAdminDocRequest {
    pub template_id: String,
    pub fields: HashMap<String, String>,
}

pub fn get_templates() -> Vec<AdminDocTemplate> {
    vec![
        // 1. CÔNG VĂN HÀNH CHÍNH
        AdminDocTemplate {
            id: "official_letter".to_string(),
            name: "Công văn Hành chính".to_string(),
            category: "Chỉ đạo & Điều hành".to_string(),
            description: "Dùng để trao đổi công việc, gửi cấp trên, đôn đốc chỉ đạo hoặc phối hợp giữa các cơ quan, đơn vị.".to_string(),
            decree_ref: "Mẫu số 01.1 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "ỦY BAN NHÂN DÂN TỈNH".to_string()),
                ("co_quan_ban_hanh".to_string(), "VĂN PHÒNG UBND TỈNH".to_string()),
                ("so_ky_hieu".to_string(), "128/VP-TH".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 15 tháng 11 năm 2026".to_string()),
                ("trich_yeu".to_string(), "V/v tăng cường công tác quản lý văn thư và chuyển đổi số năm 2026".to_string()),
                ("kinh_gui".to_string(), "Các Sở, ban, ngành thuộc tỉnh;\nỦy ban nhân dân các huyện, thành phố.".to_string()),
                ("noi_dung".to_string(), "Thực hiện Kế hoạch công tác năm 2026 của Ủy ban nhân dân tỉnh về việc đẩy mạnh ứng dụng công nghệ thông tin và chuyển đổi số trong cơ quan nhà nước;\n\nNhằm nâng cao hiệu quả công tác chỉ đạo, điều hành, xử lý văn bản và hồ sơ công việc trên môi trường điện tử, Văn phòng Ủy ban nhân dân tỉnh trân trọng đề nghị Thủ trưởng các cơ quan, đơn vị, địa phương tập trung chỉ đạo triển khai thực hiện tốt một số nội dung sau:\n\n1. Tiếp tục rà soát, hoàn thiện quy trình xử lý văn bản điện tử và ký số văn bản hành chính theo đúng quy định tại Nghị định số 30/2020/NĐ-CP của Chính phủ về công tác văn thư.\n\n2. Tăng cường khai thác, sử dụng phần mềm quản lý văn bản và điều hành; bảo đảm 100% văn bản trao đổi giữa các cơ quan nhà nước (trừ văn bản mật) được thực hiện dưới dạng văn bản điện tử.\n\n3. Định kỳ hàng quý, báo cáo kết quả thực hiện gửi về Văn phòng Ủy ban nhân dân tỉnh trước ngày 25 của tháng cuối quý để tổng hợp, báo cáo Chủ tịch Ủy ban nhân dân tỉnh.".to_string()),
                ("noi_nhan".to_string(), "- Như kính gửi;\n- Chủ tịch, các PCT UBND tỉnh (để b/c);\n- Lưu: VT, TH (3b).".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "CHÁNH VĂN PHÒNG".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },

        // 2. QUYẾT ĐỊNH
        AdminDocTemplate {
            id: "decision".to_string(),
            name: "Quyết định (Cá biệt / Trực tiếp)".to_string(),
            category: "Quyết định & Quy chế".to_string(),
            description: "Dùng để thành lập đoàn kiểm tra/thanh tra, ban hành quy chế, phân công nhiệm vụ, bổ nhiệm hoặc khen thưởng.".to_string(),
            decree_ref: "Mẫu số 01.2 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "ỦY BAN NHÂN DÂN TỈNH".to_string()),
                ("co_quan_ban_hanh".to_string(), "SỞ THÔNG TIN VÀ TRUYỀN THÔNG".to_string()),
                ("so_ky_hieu".to_string(), "86/QĐ-STTTT".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 20 tháng 11 năm 2026".to_string()),
                ("ten_quyet_dinh".to_string(), "QUYẾT ĐỊNH".to_string()),
                ("trich_yeu".to_string(), "Về việc thành lập Đoàn kiểm tra công tác an toàn thông tin và ứng dụng công nghệ thông tin năm 2026".to_string()),
                ("tham_quyen_ban_hanh".to_string(), "GIÁM ĐỐC SỞ THÔNG TIN VÀ TRUYỀN THÔNG".to_string()),
                ("can_cu".to_string(), "Căn cứ Luật Tổ chức chính quyền địa phương ngày 19 tháng 6 năm 2015;\nCăn cứ Nghị định số 30/2020/NĐ-CP ngày 05 tháng 3 năm 2020 của Chính phủ về công tác văn thư;\nCăn cứ Quyết định số 25/2021/QĐ-UBND ngày 10 tháng 8 năm 2021 của Ủy ban nhân dân tỉnh quy định chức năng, nhiệm vụ, quyền hạn và cơ cấu tổ chức của Sở Thông tin và Truyền thông;\nTheo đề nghị của Chánh Văn phòng và Trưởng phòng Công nghệ thông tin.".to_string()),
                ("noi_dung".to_string(), "QUYẾT ĐỊNH:\n\nĐiều 1. Thành lập Đoàn kiểm tra công tác an toàn thông tin và ứng dụng công nghệ thông tin năm 2026 gồm các ông (bà) có tên sau:\n1. Ông Nguyễn Văn An - Phó Giám đốc Sở, Trưởng đoàn;\n2. Bà Lê Thị Mai - Trưởng phòng CNTT, Phó Trưởng đoàn;\n3. Ông Trần Văn Nam - Chuyên viên Văn phòng, Thành viên kiêm Thư ký đoàn.\n\nĐiều 2. Nhiệm vụ và quyền hạn của Đoàn kiểm tra:\n1. Kiểm tra tình hình thực tế triển khai các hệ thống thông tin, quy trình bảo mật và văn thư điện tử tại các cơ quan, đơn vị thuộc kế hoạch kiểm tra.\n2. Lập biên bản kiểm tra, đánh giá kết quả, kịp thời hướng dẫn khắc phục tồn tại và báo cáo Giám đốc Sở sau khi kết thúc đợt kiểm tra.\n3. Thời gian kiểm tra: Từ ngày 01 tháng 12 năm 2026 đến ngày 15 tháng 12 năm 2026.\n\nĐiều 3. Chánh Văn phòng Sở, Trưởng các phòng chuyên môn thuộc Sở, Thủ trưởng các cơ quan, đơn vị liên quan và các ông (bà) có tên tại Điều 1 chịu trách nhiệm thi hành Quyết định này./.".to_string()),
                ("noi_nhan".to_string(), "- Như Điều 3;\n- UBND tỉnh (để b/c);\n- Lưu: VT, CNTT.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "GIÁM ĐỐC".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },

        // 3. TỜ TRÌNH
        AdminDocTemplate {
            id: "submission".to_string(),
            name: "Tờ trình Phê duyệt".to_string(),
            category: "Đề xuất & Báo cáo".to_string(),
            description: "Dùng để đề xuất cấp có thẩm quyền phê duyệt chủ trương, đề án, kế hoạch, kinh phí hoặc nhân sự.".to_string(),
            decree_ref: "Mẫu số 01.5 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "ỦY BAN NHÂN DÂN TỈNH".to_string()),
                ("co_quan_ban_hanh".to_string(), "SỞ NỘI VỤ".to_string()),
                ("so_ky_hieu".to_string(), "45/TTr-SNV".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 22 tháng 11 năm 2026".to_string()),
                ("trich_yeu".to_string(), "TỜ TRÌNH\nVề việc phê duyệt Kế hoạch biên chế công chức và số lượng người làm việc năm 2027".to_string()),
                ("kinh_gui".to_string(), "Ủy ban nhân dân tỉnh.".to_string()),
                ("noi_dung".to_string(), "Kính gửi: Ủy ban nhân dân tỉnh.\n\nCăn cứ Luật Cán bộ, công chức ngày 13 tháng 11 năm 2008 và Luật sửa đổi, bổ sung một số điều của Luật Cán bộ, công chức và Luật Viên chức ngày 25 tháng 11 năm 2019;\nCăn cứ Kế hoạch phát triển kinh tế - xã hội giai đoạn 2026 - 2030 của tỉnh;\n\nSở Nội vụ kính trình Ủy ban nhân dân tỉnh xem xét, phê duyệt Kế hoạch biên chế công chức và số lượng người làm việc hưởng lương từ ngân sách nhà nước năm 2027 với các nội dung chủ yếu sau:\n\nI. SỰ CẦN THIẾT VÀ CĂN CỨ PHÁP LÝ\nNhằm đáp ứng yêu cầu nâng cao hiệu lực, hiệu quả quản lý nhà nước, đồng thời tinh gọn bộ máy và tinh giản biên chế theo đúng các Nghị quyết của Trung ương...\n\nII. NỘI DUNG ĐỀ XUẤT CHÍNH\n1. Tổng số biên chế công chức đề nghị giao: 1.850 biên chế (giảm 2% so với năm 2026).\n2. Tổng số người làm việc trong các đơn vị sự nghiệp công lập: 12.400 người.\n\nIII. ĐỀ XUẤT VÀ KIẾN NGHỊ\nKính đề nghị Ủy ban nhân dân tỉnh xem xét, ký ban hành Quyết định phê duyệt theo thẩm quyền.\n(Có dự thảo Quyết định và Bảng tổng hợp chi tiết kèm theo).".to_string()),
                ("noi_nhan".to_string(), "- Như kính gửi;\n- Chủ tịch, các PCT UBND tỉnh;\n- Lưu: VT, TCBC.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "GIÁM ĐỐC".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },

        // 4. KẾ HOẠCH CÔNG TÁC
        AdminDocTemplate {
            id: "plan".to_string(),
            name: "Kế hoạch Công tác".to_string(),
            category: "Kế hoạch & Đề án".to_string(),
            description: "Dùng để xây dựng mục đích, yêu cầu, tiến độ và phân công trách nhiệm triển khai nhiệm vụ chuyên môn/hành chính.".to_string(),
            decree_ref: "Mẫu số 01.4 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "ỦY BAN NHÂN DÂN TỈNH".to_string()),
                ("co_quan_ban_hanh".to_string(), "VĂN PHÒNG ĐOÀN ĐBQH VÀ HĐND".to_string()),
                ("so_ky_hieu".to_string(), "15/KH-VP".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 25 tháng 11 năm 2026".to_string()),
                ("trich_yeu".to_string(), "KẾ HOẠCH\nCông tác rà soát, đánh giá và lập danh mục hồ sơ lưu trữ năm 2026".to_string()),
                ("noi_dung".to_string(), "I. MỤC ĐÍCH, YÊU CẦU\n1. Mục đích: Đảm bảo toàn bộ hồ sơ, tài liệu hình thành trong quá trình hoạt động của cơ quan được thu thập, phân loại, lập danh mục và giao nộp vào Lưu trữ cơ quan đúng thời hạn quy định.\n2. Yêu cầu: Việc lập danh mục phải chính xác, khoa học, đúng quy định tại Thông tư hướng dẫn và Luật Lưu trữ; không để tài liệu tồn đọng, hư hỏng.\n\nII. NỘI DUNG VÀ TIẾN ĐỘ THỰC HIỆN\n1. Rà soát, kiểm kê tài liệu từ ngày 01/12/2026 đến ngày 10/12/2026.\n2. Lập danh mục hồ sơ và bản kê chi tiết theo Mẫu số 03 từ ngày 11/12/2026 đến 20/12/2026.\n3. Tổ chức bàn giao hồ sơ lưu trữ trước ngày 30/12/2026.\n\nIII. TỔ CHỨC THỰC HIỆN\n1. Phòng Hành chính - Tổ chức: Chủ trì, hướng dẫn và đôn đốc các phòng ban chuyên môn thực hiện.\n2. Các phòng chuyên môn: Phân công công chức, viên chức trực tiếp xử lý hồ sơ tài liệu của đơn vị mình.".to_string()),
                ("noi_nhan".to_string(), "- Thường trực HĐND tỉnh (để b/c);\n- Lãnh đạo Văn phòng;\n- Các phòng chuyên môn;\n- Lưu: VT, HC-TC.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "CHÁNH VĂN PHÒNG".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },

        // 5. BÁO CÁO
        AdminDocTemplate {
            id: "report".to_string(),
            name: "Báo cáo Định kỳ / Tổng kết".to_string(),
            category: "Đề xuất & Báo cáo".to_string(),
            description: "Dùng để đánh giá kết quả thực hiện nhiệm vụ công tác tháng/quý/năm hoặc báo cáo tổng kết chuyên đề.".to_string(),
            decree_ref: "Mẫu số 01.6 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "ỦY BAN NHÂN DÂN THÀNH PHỐ".to_string()),
                ("co_quan_ban_hanh".to_string(), "ỦY BAN NHÂN DÂN QUẬN 1".to_string()),
                ("so_ky_hieu".to_string(), "102/BC-UBND".to_string()),
                ("dia_danh".to_string(), "TP. Hồ Chí Minh".to_string()),
                ("ngay_thang".to_string(), "ngày 28 tháng 11 năm 2026".to_string()),
                ("trich_yeu".to_string(), "BÁO CÁO\nKết quả thực hiện nhiệm vụ cải cách hành chính và chuyển đổi số Quý IV năm 2026".to_string()),
                ("kinh_gui".to_string(), "Ủy ban nhân dân Thành phố Hồ Chí Minh.".to_string()),
                ("noi_dung".to_string(), "I. KẾT QUẢ ĐẠT ĐƯỢC\n1. Công tác chỉ đạo, điều hành: Đã ban hành đầy đủ các văn bản hướng dẫn, kế hoạch chi tiết triển khai nhiệm vụ cải cách hành chính.\n2. Tiếp nhận và giải quyết thủ tục hành chính: Tỷ lệ giải quyết hồ sơ đúng hạn và trước hạn đạt 99.6%; 100% hồ sơ được cập nhật trên Hệ thống thông tin giải quyết thủ tục hành chính.\n3. Ứng dụng công nghệ thông tin: Triển khai hiệu quả chữ ký số cá nhân và văn bản điện tử liên thông 4 cấp.\n\nII. TỒN TẠI, HẠN CHẾ VÀ NGUYÊN NHÂN\n- Hạ tầng mạng tại một số phường đôi lúc còn chậm trong giờ cao điểm.\n- Một bộ phận người dân chưa quen sử dụng dịch vụ công trực tuyến toàn trình.\n\nIII. PHƯƠNG HƯỚNG NHIỆM VỤ TRỌNG TÂM THỜI GIAN TỚI\n1. Tiếp tục đẩy mạnh tuyên truyền, hỗ trợ người dân nộp hồ sơ trực tuyến.\n2. Tăng cường kiểm tra công vụ, kỷ luật, kỷ cương hành chính.".to_string()),
                ("noi_nhan".to_string(), "- Như kính gửi;\n- Sở Nội vụ, Sở TT&TT;\n- Chủ tịch, các PCT UBND quận;\n- Lưu: VT, CCHC.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "CHỦ TỊCH".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },

        // 6. THÔNG BÁO
        AdminDocTemplate {
            id: "notice".to_string(),
            name: "Thông báo Kết luận / Chỉ đạo".to_string(),
            category: "Chỉ đạo & Điều hành".to_string(),
            description: "Dùng để thông báo kết luận cuộc họp, ý kiến chỉ đạo, lịch nghỉ lễ, lịch trực, hoặc phân công công tác.".to_string(),
            decree_ref: "Mẫu số 01.7 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "BỘ TÀI CHÍNH".to_string()),
                ("co_quan_ban_hanh".to_string(), "CỤC THUẾ TỈNH".to_string()),
                ("so_ky_hieu".to_string(), "54/TB-CT".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 29 tháng 11 năm 2026".to_string()),
                ("trich_yeu".to_string(), "THÔNG BÁO\nÝ kiến kết luận của Cục trưởng tại cuộc họp giao ban công tác tháng 11 năm 2026".to_string()),
                ("noi_dung".to_string(), "Ngày 28 tháng 11 năm 2026, Cục Thuế đã tổ chức cuộc họp giao ban đánh giá kết quả công tác thuế tháng 11 và triển khai nhiệm vụ công tác tháng 12 năm 2026 do đồng chí Cục trưởng chủ trì.\n\nSau khi nghe báo cáo của Phòng Nghiệp vụ và ý kiến tham gia của các đại biểu tham dự cuộc họp, Cục trưởng kết luận và chỉ đạo như sau:\n\n1. Các Chi cục Thuế khu vực tập trung rà soát nguồn thu, đôn đốc thu hồi nợ đọng thuế, phấn đấu hoàn thành vượt mức dự toán thu ngân sách nhà nước năm 2026.\n\n2. Đẩy mạnh công tác hiện đại hóa ngành thuế, hỗ trợ người nộp thuế sử dụng hóa đơn điện tử khởi tạo từ máy tính tiền.\n\n3. Yêu cầu toàn thể cán bộ, công chức giữ nghiêm kỷ luật kỷ cương công vụ, văn hóa ứng xử nơi công sở.".to_string()),
                ("noi_nhan".to_string(), "- Ban Lãnh đạo Cục;\n- Các phòng thuộc Cục;\n- Các Chi cục Thuế khu vực;\n- Lưu: VT, TH.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "CỤC TRƯỞNG".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },

        // 7. BIÊN BẢN LÀM VIỆC
        AdminDocTemplate {
            id: "minutes".to_string(),
            name: "Biên bản Làm việc / Cuộc họp".to_string(),
            category: "Biên bản & Ghi nhận".to_string(),
            description: "Dùng để ghi nhận diễn biến cuộc họp, buổi làm việc chuyên môn, thanh tra, kiểm tra hoặc bàn giao tài sản.".to_string(),
            decree_ref: "Mẫu số 01.10 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "SỞ NÔNG NGHIỆP VÀ PHÁT TRIỂN NÔNG THÔN".to_string()),
                ("co_quan_ban_hanh".to_string(), "CHI CỤC KIỂM LÂM".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 30 tháng 11 năm 2026".to_string()),
                ("trich_yeu".to_string(), "BIÊN BẢN LÀM VIỆC\nVề việc kiểm tra công tác bảo vệ và phòng cháy chữa cháy rừng mùa khô 2026".to_string()),
                ("noi_dung".to_string(), "Hôm nay, vào hồi 08 giờ 30 phút ngày 30 tháng 11 năm 2026, tại Trụ sở Hạt Kiểm lâm khu vực;\n\nI. THÀNH PHẦN LÀM VIỆC:\n1. Đoàn kiểm tra Chi cục Kiểm lâm:\n- Ông Nguyễn Văn An - Phó Chi cục trưởng, Trưởng đoàn;\n- Bà Trần Thị Lan - Chuyên viên, Thư ký.\n\n2. Đại diện đơn vị được kiểm tra:\n- Ông Hoàng Văn Bình - Hạt trưởng Hạt Kiểm lâm;\n- Các cán bộ phụ trách địa bàn.\n\nII. NỘI DUNG VÀ KẾT QUẢ KIỂM TRA:\n1. Tình hình trang thiết bị PCCC rừng: Đầy đủ máy thổi gió, máy bơm nước, hệ thống thông tin liên lạc sẵn sàng chiến đấu.\n2. Lực lượng thường trực: Bố trí lực lượng ứng trực 24/24 giờ tại các chòi canh lửa trọng điểm.\n3. Kết luận: Đơn vị đã chủ động thực hiện tốt phương án PCCC rừng. Yêu cầu tiếp tục theo dõi sát diễn biến thời tiết.\n\nBiên bản kết thúc vào hồi 11 giờ 00 cùng ngày, đã được đọc lại cho các thành viên cùng nghe và thống nhất ký tên.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "TRƯỞNG ĐOÀN KIỂM TRA".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
                ("chuc_vu_ben_b".to_string(), "ĐẠI DIỆN ĐƠN VỊ ĐƯỢC KIỂM TRA".to_string()),
                ("ho_ten_ben_b".to_string(), "Hoàng Văn Bình".to_string()),
            ]),
        },

        // 8. GIẤY MỜI
        AdminDocTemplate {
            id: "invitation".to_string(),
            name: "Giấy mời Họp / Hội nghị".to_string(),
            category: "Chỉ đạo & Điều hành".to_string(),
            description: "Dùng để mời họp giao ban, hội nghị tổng kết, hội thảo chuyên đề hoặc dự làm việc.".to_string(),
            decree_ref: "Mẫu số 01.12 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "ỦY BAN NHÂN DÂN HUYỆN".to_string()),
                ("co_quan_ban_hanh".to_string(), "VĂN PHÒNG HĐND VÀ UBND".to_string()),
                ("so_ky_hieu".to_string(), "24/GM-VP".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 01 tháng 12 năm 2026".to_string()),
                ("trich_yeu".to_string(), "GIẤY MỜI\nTham dự Hội nghị tổng kết công tác cải cách hành chính năm 2026".to_string()),
                ("kinh_gui".to_string(), "Thủ trưởng các phòng, ban, ngành, đoàn thể huyện;\nChủ tịch Ủy ban nhân dân các xã, thị trấn.".to_string()),
                ("noi_dung".to_string(), "Văn phòng HĐND và UBND huyện trân trọng kính mời các đồng chí tham dự Hội nghị tổng kết công tác cải cách hành chính năm 2026 và triển khai phương hướng nhiệm vụ năm 2027.\n\n1. Thời gian: Bắt đầu từ 08 giờ 00 phút, ngày 05 tháng 12 năm 2026 (thứ Bảy).\n2. Địa điểm: Hội trường lớn Ủy ban nhân dân huyện (Tầng 3, Trụ sở UBND huyện).\n3. Chủ trì: Đồng chí Chủ tịch Ủy ban nhân dân huyện.\n\nYêu cầu các đại biểu tham dự đúng giờ, nghiên cứu tài liệu đã gửi trên Hệ thống văn bản điện tử và chuẩn bị ý kiến phát biểu.".to_string()),
                ("noi_nhan".to_string(), "- Như kính gửi;\n- Thường trực Huyện ủy;\n- Lưu: VT, TH.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "CHÁNH VĂN PHÒNG".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },

        // 9. GIẤY GIỚI THIỆU
        AdminDocTemplate {
            id: "letter_of_introduction".to_string(),
            name: "Giấy giới thiệu Công tác".to_string(),
            category: "Hành chính & Cán bộ".to_string(),
            description: "Dùng để giới thiệu cán bộ, công chức đến cơ quan, đơn vị khác liên hệ công tác hoặc xác minh nghiệp vụ.".to_string(),
            decree_ref: "Mẫu số 01.13 Phụ lục III Nghị định 30/2020/NĐ-CP".to_string(),
            default_fields: HashMap::from([
                ("co_quan_chu_quan".to_string(), "CÔNG AN TỈNH".to_string()),
                ("co_quan_ban_hanh".to_string(), "PHÒNG HỒ SƠ NGHIỆP VỤ".to_string()),
                ("so_ky_hieu".to_string(), "18/GGT-PV06".to_string()),
                ("dia_danh".to_string(), "Hà Nội".to_string()),
                ("ngay_thang".to_string(), "ngày 02 tháng 12 năm 2026".to_string()),
                ("trich_yeu".to_string(), "GIẤY GIỚI THIỆU".to_string()),
                ("kinh_gui".to_string(), "Văn phòng Ủy ban nhân dân tỉnh.".to_string()),
                ("noi_dung".to_string(), "Phòng Hồ sơ nghiệp vụ trân trọng giới thiệu:\n\nĐồng chí: Nguyễn Văn An.\nChức vụ: Cán bộ chuyên trách quản lý hồ sơ và chuyển đổi số.\nSố Căn cước công dân: 001095012345, cấp ngày 15/04/2023 tại Cục Cảnh sát QLHC về TTXH.\n\nĐược cử đến: Văn phòng Ủy ban nhân dân tỉnh.\nVề việc: Phối hợp số hóa, đồng bộ dữ liệu và tra cứu văn bản phục vụ nghiệp vụ công tác.\n\nGiấy giới thiệu này có giá trị đến hết ngày 31 tháng 12 năm 2026.".to_string()),
                ("chuc_vu_nguoi_ky".to_string(), "TRƯỞNG PHÒNG".to_string()),
                ("ho_ten_nguoi_ky".to_string(), "Nguyễn Văn An".to_string()),
            ]),
        },
    ]
}

pub fn generate_admin_docx(template_id: &str, fields: &HashMap<String, String>) -> Result<Vec<u8>, String> {
    let mut file = Cursor::new(Vec::new());
    let mut docx = Docx::new();

    // Standard A4 Margins per Decree 30/2020/ND-CP:
    // Top: 20mm (1134 dxa), Bottom: 20mm (1134 dxa), Left: 30mm (1701 dxa), Right: 15mm (850 dxa)
    docx = docx.page_margin(PageMargin::new().top(1134).bottom(1134).left(1701).right(850));

    let empty = String::new();
    let get_val = |k: &str| fields.get(k).unwrap_or(&empty).as_str();

    let co_quan_chu_quan = get_val("co_quan_chu_quan");
    let co_quan_ban_hanh = get_val("co_quan_ban_hanh");
    let so_ky_hieu = get_val("so_ky_hieu");
    let dia_danh = get_val("dia_danh");
    let ngay_thang = get_val("ngay_thang");
    let trich_yeu = get_val("trich_yeu");
    let kinh_gui = get_val("kinh_gui");
    let noi_dung = get_val("noi_dung");
    let noi_nhan = get_val("noi_nhan");
    let chuc_vu = get_val("chuc_vu_nguoi_ky");
    let ho_ten = get_val("ho_ten_nguoi_ky");

    // ==========================================
    // 1. HEADER TABLE (2 CELLS: CƠ QUAN vs QUỐC HIỆU)
    // ==========================================
    let mut left_cell = TableCell::new();
    if !co_quan_chu_quan.is_empty() {
        left_cell = left_cell.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(co_quan_chu_quan).size(24).fonts(RunFonts::new().ascii("Times New Roman")))
        );
    }
    if !co_quan_ban_hanh.is_empty() {
        left_cell = left_cell.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(co_quan_ban_hanh).bold().size(24).fonts(RunFonts::new().ascii("Times New Roman")))
        );
        // Short underline separator for issuing agency
        left_cell = left_cell.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text("──────────").size(18).fonts(RunFonts::new().ascii("Times New Roman")))
        );
    }
    if !so_ky_hieu.is_empty() {
        left_cell = left_cell.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(format!("Số: {}", so_ky_hieu)).size(24).fonts(RunFonts::new().ascii("Times New Roman")))
        );
    }

    let mut right_cell = TableCell::new();
    right_cell = right_cell.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text("CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM").bold().size(24).fonts(RunFonts::new().ascii("Times New Roman")))
    );
    right_cell = right_cell.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text("Độc lập - Tự do - Hạnh phúc").bold().size(26).fonts(RunFonts::new().ascii("Times New Roman")))
    );
    right_cell = right_cell.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text("──────────────").size(18).fonts(RunFonts::new().ascii("Times New Roman")))
    );
    if !dia_danh.is_empty() && !ngay_thang.is_empty() {
        right_cell = right_cell.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(format!("{}, {}", dia_danh, ngay_thang)).italic().size(26).fonts(RunFonts::new().ascii("Times New Roman")))
        );
    }

    let header_row = TableRow::new(vec![left_cell, right_cell]);
    let header_table = Table::new(vec![header_row]).set_borders(TableBorders::new().clear_all());
    docx = docx.add_table(header_table);
    docx = docx.add_paragraph(Paragraph::new()); // blank spacing line

    // ==========================================
    // 2. DOCUMENT TITLE & TRÍCH YẾU
    // ==========================================
    match template_id {
        "official_letter" => {
            // For Official Letter: Trích yếu is on left under Số: V/v...
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(trich_yeu).italic().size(24).fonts(RunFonts::new().ascii("Times New Roman")))
            );
            docx = docx.add_paragraph(Paragraph::new());
            if !kinh_gui.is_empty() {
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .align(AlignmentType::Center)
                        .add_run(Run::new().add_text(format!("Kính gửi: {}", kinh_gui)).bold().size(28).fonts(RunFonts::new().ascii("Times New Roman")))
                );
                docx = docx.add_paragraph(Paragraph::new());
            }
        },
        _ => {
            // For Decisions, Plans, Reports, Minutes, Notices, Submissions:
            let title_lines: Vec<&str> = trich_yeu.split('\n').collect();
            for (idx, line) in title_lines.iter().enumerate() {
                let p = Paragraph::new()
                    .align(AlignmentType::Center)
                    .add_run(
                        Run::new()
                            .add_text(line.trim())
                            .bold()
                            .size(if idx == 0 { 30 } else { 28 }) // 15pt / 14pt
                            .fonts(RunFonts::new().ascii("Times New Roman"))
                    );
                docx = docx.add_paragraph(p);
            }
            docx = docx.add_paragraph(Paragraph::new());

            if !kinh_gui.is_empty() {
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .align(AlignmentType::Center)
                        .add_run(Run::new().add_text(format!("Kính gửi: {}", kinh_gui)).bold().size(28).fonts(RunFonts::new().ascii("Times New Roman")))
                );
                docx = docx.add_paragraph(Paragraph::new());
            }
        }
    }

    // ==========================================
    // 3. BODY TEXT (CHUẨN THỂ THỨC NGHỊ ĐỊNH 30)
    // ==========================================
    for paragraph_str in noi_dung.split("\n\n") {
        let trimmed = paragraph_str.trim();
        if trimmed.is_empty() { continue; }

        let mut p = Paragraph::new()
            .align(AlignmentType::Both) // Canh đều 2 bên (Justified)
            .line_spacing(LineSpacing::new().line(276)); // 1.15 line spacing

        // Check if heading or standard body
        if trimmed.starts_with("Điều ") || trimmed.starts_with("I. ") || trimmed.starts_with("II. ") || trimmed.starts_with("III. ") || trimmed.starts_with("QUYẾT ĐỊNH:") {
            p = p.add_run(Run::new().add_text(trimmed).bold().size(28).fonts(RunFonts::new().ascii("Times New Roman")));
        } else {
            p = p.indent(Some(567), None, None, None) // 1 cm first-line indent (567 dxa)
                 .add_run(Run::new().add_text(trimmed).size(28).fonts(RunFonts::new().ascii("Times New Roman")));
        }
        docx = docx.add_paragraph(p);
    }
    docx = docx.add_paragraph(Paragraph::new());

    // ==========================================
    // 4. FOOTER TABLE (NƠI NHẬN & CHỮ KÝ)
    // ==========================================
    let mut f_left = TableCell::new();
    if !noi_nhan.is_empty() {
        f_left = f_left.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Nơi nhận:").bold().italic().size(24).fonts(RunFonts::new().ascii("Times New Roman")))
        );
        for line in noi_nhan.lines() {
            let l_trimmed = line.trim();
            if l_trimmed.is_empty() || l_trimmed == "Nơi nhận:" { continue; }
            f_left = f_left.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(l_trimmed).italic().size(22).fonts(RunFonts::new().ascii("Times New Roman")))
            );
        }
    }

    let mut f_right = TableCell::new();
    if !chuc_vu.is_empty() {
        f_right = f_right.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(chuc_vu).bold().size(28).fonts(RunFonts::new().ascii("Times New Roman")))
        );
        f_right = f_right.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text("(Ký, ghi rõ họ tên và đóng dấu)").italic().size(22).fonts(RunFonts::new().ascii("Times New Roman")))
        );
        // Signature spacing
        f_right = f_right.add_paragraph(Paragraph::new());
        f_right = f_right.add_paragraph(Paragraph::new());
        f_right = f_right.add_paragraph(Paragraph::new());
        f_right = f_right.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(ho_ten).bold().size(28).fonts(RunFonts::new().ascii("Times New Roman")))
        );
    }

    let footer_row = TableRow::new(vec![f_left, f_right]);
    let footer_table = Table::new(vec![footer_row]).set_borders(TableBorders::new().clear_all());
    docx = docx.add_table(footer_table);

    docx.build().pack(&mut file).map_err(|e| format!("Lỗi xuất DOCX: {:?}", e))?;
    Ok(file.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_templates() {
        let templates = get_templates();
        assert_eq!(templates.len(), 9);
        for t in &templates {
            assert!(!t.id.is_empty());
            assert!(!t.name.is_empty());
            assert!(!t.default_fields.is_empty());
        }
    }

    #[test]
    fn test_generate_admin_docx_all_templates() {
        let templates = get_templates();
        for t in &templates {
            let res = generate_admin_docx(&t.id, &t.default_fields);
            assert!(res.is_ok(), "Failed for template {}: {:?}", t.id, res.err());
            let bytes = res.unwrap();
            assert!(!bytes.is_empty(), "Empty bytes for template {}", t.id);
            // Verify DOCX ZIP magic bytes "PK\x03\x04"
            assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
        }
    }
}

