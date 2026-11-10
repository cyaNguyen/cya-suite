use futures_util::{SinkExt, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

const TRUSTED_CLIENT_TOKEN: &str = "6A5AA1D4EA6511CF84E696E812D3A7BF";
const WSS_URL: &str = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceOption {
    pub id: String,
    pub name: String,
    pub gender: String,
}

pub fn get_available_voices() -> Vec<VoiceOption> {
    vec![
        VoiceOption {
            id: "vi-VN-HoaiMyNeural".to_string(),
            name: "Hoài My (Nữ - Truyền cảm, tự nhiên)".to_string(),
            gender: "Nữ".to_string(),
        },
        VoiceOption {
            id: "vi-VN-NamMinhNeural".to_string(),
            name: "Nam Minh (Nam - Trầm ấm, dứt khoát)".to_string(),
            gender: "Nam".to_string(),
        },
    ]
}

/// Chuyển văn bản thành giọng nói (Edge TTS) thuần Rust
pub async fn synthesize_speech(
    text: &str,
    voice: &str,
    rate_percent: i32,
    output_mp3_path: &Path,
) -> Result<Vec<u8>, String> {
    let connection_id = Uuid::new_v4().simple().to_string();
    let url_str = format!("{}?TrustedClientToken={}&ConnectionId={}", WSS_URL, TRUSTED_CLIENT_TOKEN, connection_id);

    let mut request = url_str.into_client_request().map_err(|e| e.to_string())?;
    let headers = request.headers_mut();
    headers.insert("Origin", "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold".parse().unwrap());
    headers.insert("Pragma", "no-cache".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0".parse().unwrap());

    let (ws_stream, _) = connect_async(request).await.map_err(|e| format!("Lỗi kết nối Edge TTS: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // 1. Gửi cấu hình định dạng audio
    let config_msg = format!(
        "Content-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"}},\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"}}}}}}}}"
    );
    write.send(Message::Text(config_msg.into())).await.map_err(|e| e.to_string())?;

    // 2. Chuẩn bị SSML
    let rate_str = if rate_percent >= 0 {
        format!("+{}%", rate_percent)
    } else {
        format!("{}%", rate_percent)
    };

    let req_id = Uuid::new_v4().simple().to_string();
    let ssml = format!(
        "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nPath:ssml\r\n\r\n<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='vi-VN'><voice name='{}'><prosody pitch='+0Hz' rate='{}'>{}</prosody></voice></speak>",
        req_id,
        voice,
        rate_str,
        quick_xml_escape(text)
    );
    write.send(Message::Text(ssml.into())).await.map_err(|e| e.to_string())?;

    let mut audio_data = Vec::new();

    // 3. Đọc dữ liệu audio stream
    while let Some(msg_res) = read.next().await {
        let msg = msg_res.map_err(|e| e.to_string())?;
        match msg {
            Message::Binary(bin) => {
                if bin.len() > 2 {
                    let header_len = u16::from_be_bytes([bin[0], bin[1]]) as usize;
                    if bin.len() > 2 + header_len {
                        let header_str = String::from_utf8_lossy(&bin[2..2 + header_len]);
                        if header_str.contains("Path:audio") {
                            audio_data.extend_from_slice(&bin[2 + header_len..]);
                        }
                    }
                }
            }
            Message::Text(txt) => {
                if txt.contains("Path:turn.end") {
                    break;
                }
            }
            _ => {}
        }
    }

    if audio_data.is_empty() {
        return Err("Không nhận được dữ liệu âm thanh từ máy chủ Microsoft".into());
    }

    let mut file = File::create(output_mp3_path).map_err(|e| e.to_string())?;
    file.write_all(&audio_data).map_err(|e| e.to_string())?;

    Ok(audio_data)
}

/// Tạo phụ đề SRT chuẩn từ nội dung văn bản và tổng thời lượng âm thanh
pub fn generate_srt(text: &str, total_seconds: f64, srt_path: &Path) -> std::io::Result<()> {
    let re = Regex::new(r"(?i)(?<=[.!?。！？\n])\s+").unwrap();
    let raw_sentences: Vec<&str> = re.split(text).map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if raw_sentences.is_empty() {
        return Ok(());
    }

    let total_chars: usize = raw_sentences.iter().map(|s| s.chars().count()).sum();
    let mut current_sec = 0.0;
    let mut srt_content = String::new();

    for (i, sentence) in raw_sentences.iter().enumerate() {
        let chars = sentence.chars().count();
        let duration = if total_chars > 0 {
            (chars as f64 / total_chars as f64) * total_seconds
        } else {
            total_seconds / raw_sentences.len() as f64
        };

        let start_time = current_sec;
        let end_time = (current_sec + duration).min(total_seconds);
        current_sec = end_time;

        srt_content.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            i + 1,
            format_srt_time(start_time),
            format_srt_time(end_time),
            sentence
        ));
    }

    let mut file = File::create(srt_path)?;
    file.write_all(srt_content.as_bytes())?;
    Ok(())
}

fn format_srt_time(sec: f64) -> String {
    let total_ms = (sec * 1000.0) as u64;
    let ms = total_ms % 1000;
    let total_s = total_ms / 1000;
    let s = total_s % 60;
    let total_m = total_s / 60;
    let m = total_m % 60;
    let h = total_m / 60;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

fn quick_xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
