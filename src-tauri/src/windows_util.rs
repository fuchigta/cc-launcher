use std::os::windows::process::CommandExt;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn no_window_command(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// マウスカーソルが位置するモニタのワークエリア (x, y, width, height) を返す
pub fn get_cursor_monitor_work_area() -> Option<(i32, i32, i32, i32)> {
    unsafe {
        let mut cursor_pos = POINT { x: 0, y: 0 };
        GetCursorPos(&mut cursor_pos).ok()?;

        let monitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTONEAREST);
        if monitor.is_invalid() {
            return None;
        }

        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            let rc = info.rcWork;
            Some((rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top))
        } else {
            None
        }
    }
}

/// デフォルト作業ディレクトリ（%USERPROFILE%\cc-launcher）を取得・作成
pub fn default_working_dir() -> Option<std::path::PathBuf> {
    let dir = dirs::home_dir()?.join("cc-launcher");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}
