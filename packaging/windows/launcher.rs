#![windows_subsystem = "windows"]

use std::env;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::ptr;
use std::thread;
use std::time::Duration;

#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(
        window: *mut core::ffi::c_void,
        text: *const u16,
        caption: *const u16,
        kind: u32,
    ) -> i32;
}

const MB_OK: u32 = 0;
const MB_ICONERROR: u32 = 0x10;

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}

fn show_error(message: &str) {
    let message = wide(OsStr::new(message));
    let caption = wide(OsStr::new("Waylyrics could not start"));
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            message.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn portable_path(root: &Path, relative: &str) -> std::path::PathBuf {
    root.join(relative)
}

fn main() {
    let executable = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            show_error(&format!("Could not locate the portable folder.\n\n{error}"));
            return;
        }
    };
    let Some(root) = executable.parent() else {
        show_error("Could not locate the portable folder.");
        return;
    };

    let app = portable_path(root, "bin\\waylyrics-app.exe");
    if !app.is_file() {
        show_error(
            "The portable package is incomplete.\n\nMissing: bin\\waylyrics-app.exe\n\nPlease extract the entire ZIP before running Waylyrics.",
        );
        return;
    }

    let bin = portable_path(root, "bin");
    let mut path = bin.clone().into_os_string();
    path.push(";");
    path.push(root.as_os_str());
    if let Some(existing) = env::var_os("PATH") {
        path.push(";");
        path.push(existing);
    }

    let log_path = portable_path(root, "waylyrics-startup.log");
    let mut command = Command::new(&app);
    command
        .current_dir(root)
        .env("WAYLYRICS_THEME_PRESETS_DIR", portable_path(root, "themes"))
        .env("XDG_DATA_DIRS", portable_path(root, "share"))
        .env(
            "GSETTINGS_SCHEMA_DIR",
            portable_path(root, "share\\glib-2.0\\schemas"),
        )
        .env(
            "GDK_PIXBUF_MODULE_FILE",
            portable_path(root, "lib\\gdk-pixbuf-2.0\\2.10.0\\loaders.cache"),
        )
        .env("PATH", path);

    if let Ok(mut log) = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&log_path)
    {
        let _ = writeln!(log, "Waylyrics portable startup log");
        let _ = writeln!(log, "Application: {}", app.display());
        let _ = writeln!(log);
        if let Ok(error_log) = log.try_clone() {
            command
                .stdout(Stdio::from(log))
                .stderr(Stdio::from(error_log));
        }
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            show_error(&format!(
                "Waylyrics could not start.\n\n{error}\n\nPlease extract the entire ZIP before running the app."
            ));
            return;
        }
    };

    thread::sleep(Duration::from_secs(5));
    match child.try_wait() {
        Ok(Some(status)) => {
            let code = status
                .code()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "terminated by Windows or security software".to_owned());
            show_error(&format!(
                "Waylyrics exited during startup.\n\nExit status: {code}\n\nDiagnostic log:\n{}\n\nIf Waylyrics is not already running, please send this log file when reporting the problem.",
                log_path.display()
            ));
        }
        Ok(None) => {}
        Err(error) => show_error(&format!(
            "Waylyrics started, but its status could not be checked.\n\n{error}\n\nDiagnostic log:\n{}",
            log_path.display()
        )),
    }
}
