use std::process::Command;

/// Chuẩn hóa và loại bỏ dấu tiếng Việt để tìm kiếm không dấu siêu tốc
pub fn remove_accents(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        let mapped = match c {
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => 'a',
            'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ' | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'A',
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'e',
            'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'E',
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' => 'o',
            'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ' | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => 'O',
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'u',
            'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'U',
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
            'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'Y',
            'đ' => 'd',
            'Đ' => 'D',
            other => other,
        };
        result.push(mapped);
    }
    result.to_lowercase()
}

/// Mở file hoặc thư mục trên Windows an toàn
pub fn open_file_or_folder(path: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(path).spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

/// Mở thư mục chứa file và bôi đen file đó trong Windows Explorer
pub fn open_containing_folder(path: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let clean_path = path.replace('/', "\\");
        Command::new("explorer")
            .args(["/select,", &clean_path])
            .spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(parent) = std::path::Path::new(path).parent() {
            Command::new("xdg-open").arg(parent).spawn()?;
        }
    }
    Ok(())
}

/// Lấy danh sách các ổ đĩa thực tế trên máy tính (C:\, D:\, E:\...)
pub fn get_system_drives() -> Vec<String> {
    let mut drives = Vec::new();
    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let drive_str = format!("{}:\\", letter as char);
            if std::path::Path::new(&drive_str).exists() {
                drives.push(drive_str);
            }
        }
    }
    if drives.is_empty() {
        drives.push("C:\\".into());
    }
    drives
}

/// Mở cửa sổ chọn thư mục native Windows
pub fn pick_folder_native() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Dùng Shell.Application.BrowseForFolder (Native Windows API, luôn nổi lên desktop HWND 0)
        let ps_cmd = r#"$sh = New-Object -ComObject Shell.Application; $f = $sh.BrowseForFolder(0, 'Chọn thư mục chứa tài liệu', 81, 0); if ($f) { Write-Output $f.Self.Path }"#;
        if let Ok(out) = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Sta", "-Command", ps_cmd])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }

        // 2. Fallback WinForms FolderBrowserDialog chuẩn
        let ps_winforms = r#"[System.Reflection.Assembly]::LoadWithPartialName('System.windows.forms') | Out-Null; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.AutoUpgradeEnabled = $true; $f.Description = 'Chọn thư mục chứa tài liệu'; if ($f.ShowDialog() -eq 'OK') { Write-Output $f.SelectedPath }"#;
        if let Ok(out) = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Sta", "-Command", ps_winforms])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
    }

    // 3. Fallback qua rfd
    rfd::FileDialog::new()
        .set_title("Chọn thư mục chứa tài liệu")
        .pick_folder()
        .map(|p| p.to_string_lossy().to_string())
}
