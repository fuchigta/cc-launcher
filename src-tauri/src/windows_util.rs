use std::os::windows::process::CommandExt;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn no_window_command(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// デフォルト作業ディレクトリ（%USERPROFILE%\cc-launcher）を取得・作成
pub fn default_working_dir() -> Option<std::path::PathBuf> {
    let dir = dirs::home_dir()?.join("cc-launcher");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}
