use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent, TrayIcon},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const WINDOW_WIDTH: f64 = 340.0;
const TASKS_FILE: &str = "tasks.json";
const DATA_DIR: &str = "com.desktodo.desk";

/// Task data structure matching the frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub text: String,
    pub completed: bool,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub color: String,
    pub ddl: Option<u64>,
}

#[derive(Default)]
struct TasksState {
    items: Mutex<Vec<Task>>,
    data_dir: Mutex<Option<PathBuf>>,
}

struct AppState {
    tray: Mutex<Option<TrayIcon>>,
    recording: Mutex<bool>,
    shortcut_mods: Mutex<Vec<String>>,
    shortcut_key: Mutex<String>,
    /// Set to true when window is being shown; cleared on Focused(true).
    /// While true, Focused(false) events are ignored (they come from the
    /// show+focus transition, not from the user clicking away).
    just_shown: Arc<Mutex<bool>>,
    tasks: TasksState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tray: Mutex::new(None),
            recording: Mutex::new(false),
            shortcut_mods: Mutex::new(vec!["Control".to_string()]),
            shortcut_key: Mutex::new("2".to_string()),
            just_shown: Arc::new(Mutex::new(false)),
            tasks: TasksState::default(),
        }
    }
}

/// Get the path to the tasks JSON file
fn tasks_file_path(app: &AppHandle) -> Option<PathBuf> {
    let state = app.state::<AppState>();
    let guard = state.tasks.data_dir.lock().unwrap();
    guard.as_ref().map(|p| p.join(TASKS_FILE))
}

/// Synchronously save tasks to disk
fn persist_tasks(app: &AppHandle) {
    if let Some(path) = tasks_file_path(app) {
        let state = app.state::<AppState>();
        let tasks = state.tasks.items.lock().unwrap();
        if let Ok(json) = serde_json::to_string_pretty(&*tasks) {
            let _ = fs::write(&path, json);
        }
    }
}

#[tauri::command]
fn load_tasks(app: AppHandle) -> Vec<Task> {
    if let Some(path) = tasks_file_path(&app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(tasks) = serde_json::from_str::<Vec<Task>>(&content) {
                    let state = app.state::<AppState>();
                    *state.tasks.items.lock().unwrap() = tasks.clone();
                    return tasks;
                }
            }
        }
    }
    Vec::new()
}

#[tauri::command]
fn add_task(app: AppHandle, text: String, ddl: Option<u64>) -> Task {
    let color = pick_color(&text);
    let task = Task {
        id: uuid_v4(),
        text,
        completed: false,
        created_at: current_timestamp(),
        color,
        ddl,
    };
    {
        let state = app.state::<AppState>();
        state.tasks.items.lock().unwrap().insert(0, task.clone());
    }
    persist_tasks(&app);
    let _ = app.emit("tasks-changed", app.state::<AppState>().tasks.items.lock().unwrap().clone());
    task
}

#[tauri::command]
fn toggle_task(app: AppHandle, id: String) {
    {
        let state = app.state::<AppState>();
        let mut tasks = state.tasks.items.lock().unwrap();
        if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
            t.completed = !t.completed;
        }
    }
    persist_tasks(&app);
    let _ = app.emit("tasks-changed", app.state::<AppState>().tasks.items.lock().unwrap().clone());
}

#[tauri::command]
fn delete_task(app: AppHandle, id: String) {
    {
        let state = app.state::<AppState>();
        state.tasks.items.lock().unwrap().retain(|t| t.id != id);
    }
    persist_tasks(&app);
    let _ = app.emit("tasks-changed", app.state::<AppState>().tasks.items.lock().unwrap().clone());
}

// ── Small utility helpers ─────────────────────────────────────────────

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    let hash = secs.wrapping_mul(0x9e3779b9).wrapping_add(nanos as u64);
    let bytes: [u8; 16] = [
        (hash as u8), ((hash >> 8) as u8), ((hash >> 16) as u8), ((hash >> 24) as u8),
        ((hash >> 32) as u8), ((hash >> 40) as u8), ((hash >> 48) as u8), ((hash >> 56) as u8),
        (!hash as u8), ((!hash >> 8) as u8), ((!hash >> 16) as u8), ((!hash >> 24) as u8),
        ((!hash >> 32) as u8), ((!hash >> 40) as u8), ((!hash >> 48) as u8), ((!hash >> 56) as u8),
    ];
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

const TASK_COLORS: [&str; 8] = [
    "#0a84ff", "#30d158", "#ff9f0a", "#bf5af2",
    "#ff375f", "#64d2ff", "#ffd60a", "#ac8e68",
];

fn pick_color(text: &str) -> String {
    let mut hash: i64 = 0;
    for b in text.bytes() {
        hash = (b as i64).wrapping_add(hash.wrapping_mul(31));
    }
    TASK_COLORS[hash.abs() as usize % TASK_COLORS.len()].to_string()
}

fn show_window_at(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(r) = state.recording.lock() {
                    if *r {
                        return;
                    }
                }
            }
            let _ = window.hide();
            return;
        }

        // Mark as "just shown" so blur events during focus transition are ignored
        if let Some(state) = app.try_state::<AppState>() {
            let js = state.just_shown.clone();
            *js.lock().unwrap() = true;
            // Safety timeout: clear flag after 2s in case Focused(true) never arrives
            let js2 = js.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(2));
                *js2.lock().unwrap() = false;
            });
        }

        if let Ok(monitor) = window.primary_monitor() {
            if let Some(monitor) = monitor {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let logical_w = size.width as f64 / scale;
                let x = logical_w - WINDOW_WIDTH - 16.0;
                let y = 28.0 + 8.0;
                let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
            }
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn build_shortcut(modifiers: &[String], key: &str) -> Option<Shortcut> {
    let mods: Vec<Modifiers> = modifiers
        .iter()
        .filter_map(|m| match m.as_str() {
            "Control" | "Ctrl" => Some(Modifiers::CONTROL),
            "Option" | "Alt" => Some(Modifiers::ALT),
            "Shift" => Some(Modifiers::SHIFT),
            "Super" | "Cmd" | "Command" | "Meta" => Some(Modifiers::SUPER),
            _ => None,
        })
        .collect();

    let code = match key.to_uppercase().as_str() {
        "SPACE" => Code::Space,
        "A" => Code::KeyA, "B" => Code::KeyB, "C" => Code::KeyC, "D" => Code::KeyD,
        "E" => Code::KeyE, "F" => Code::KeyF, "G" => Code::KeyG, "H" => Code::KeyH,
        "I" => Code::KeyI, "J" => Code::KeyJ, "K" => Code::KeyK, "L" => Code::KeyL,
        "M" => Code::KeyM, "N" => Code::KeyN, "O" => Code::KeyO, "P" => Code::KeyP,
        "Q" => Code::KeyQ, "R" => Code::KeyR, "S" => Code::KeyS, "T" => Code::KeyT,
        "U" => Code::KeyU, "V" => Code::KeyV, "W" => Code::KeyW, "X" => Code::KeyX,
        "Y" => Code::KeyY, "Z" => Code::KeyZ,
        "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2, "3" => Code::Digit3,
        "4" => Code::Digit4, "5" => Code::Digit5, "6" => Code::Digit6, "7" => Code::Digit7,
        "8" => Code::Digit8, "9" => Code::Digit9,
        "F1" => Code::F1, "F2" => Code::F2, "F3" => Code::F3, "F4" => Code::F4,
        "F5" => Code::F5, "F6" => Code::F6, "F7" => Code::F7, "F8" => Code::F8,
        "F9" => Code::F9, "F10" => Code::F10, "F11" => Code::F11, "F12" => Code::F12,
        "ENTER" | "RETURN" => Code::Enter,
        "TAB" => Code::Tab,
        "ESCAPE" | "ESC" => Code::Escape,
        "BACKSPACE" => Code::Backspace,
        "DELETE" => Code::Delete,
        "UP" => Code::ArrowUp, "DOWN" => Code::ArrowDown,
        "LEFT" => Code::ArrowLeft, "RIGHT" => Code::ArrowRight,
        _ => return None,
    };

    Some(Shortcut::new(
        if mods.is_empty() {
            None
        } else {
            Some(mods.iter().fold(Modifiers::empty(), |acc, m| acc | *m))
        },
        code,
    ))
}

fn format_label(modifiers: &[String], key: &str) -> String {
    let parts: Vec<&str> = modifiers
        .iter()
        .map(|m| match m.as_str() {
            "Control" | "Ctrl" => "Ctrl",
            "Option" | "Alt" => "Option",
            "Shift" => "Shift",
            "Super" | "Cmd" | "Command" | "Meta" => "Cmd",
            _ => m.as_str(),
        })
        .collect();
    let mut all: Vec<&str> = parts;
    all.push(key);
    all.join(" + ")
}

fn register_shortcut(app: &AppHandle, modifiers: Vec<String>, key: String) {
    if let Some(shortcut) = build_shortcut(&modifiers, &key) {
        let _ = app.global_shortcut().on_shortcut(shortcut, |app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                show_window_at(app);
            }
        });
    }
}

#[tauri::command]
fn update_shortcut(
    app: AppHandle,
    state: State<'_, AppState>,
    modifiers: Vec<String>,
    key: String,
) -> Result<String, String> {
    let shortcut = build_shortcut(&modifiers, &key)
        .ok_or_else(|| "Unsupported key combination".to_string())?;

    let _ = app.global_shortcut().unregister_all();
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                show_window_at(app);
            }
        })
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    if let Ok(mut m) = state.shortcut_mods.lock() {
        *m = modifiers.clone();
    }
    if let Ok(mut k) = state.shortcut_key.lock() {
        *k = key.clone();
    }

    let label = format_label(&modifiers, &key);

    if let Ok(tray_guard) = state.tray.lock() {
        if let Some(ref tray) = *tray_guard {
            let _ = tray.set_tooltip(Some(&format!("ToDo  ({})", label)));
        }
    }

    Ok(label)
}

#[tauri::command]
fn get_current_shortcut(state: State<'_, AppState>) -> String {
    let mods = state.shortcut_mods.lock().unwrap();
    let key = state.shortcut_key.lock().unwrap();
    format_label(&mods, &key)
}

#[tauri::command]
fn start_recording(state: State<'_, AppState>) {
    if let Ok(mut r) = state.recording.lock() {
        *r = true;
    }
}

#[tauri::command]
fn stop_recording(state: State<'_, AppState>) {
    if let Ok(mut r) = state.recording.lock() {
        *r = false;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            update_shortcut,
            get_current_shortcut,
            start_recording,
            stop_recording,
            load_tasks,
            add_task,
            toggle_task,
            delete_task,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Window focus event handler
            let just_shown = app.state::<AppState>().just_shown.clone();
            let w = app.get_webview_window("main").unwrap();
            let window_for_event = w.clone();
            w.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Focused(true) => {
                        // Window gained focus — delay clearing just_shown so that
                        // any Focused(false) that follows immediately is still suppressed
                        let js2 = just_shown.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(500));
                            *js2.lock().unwrap() = false;
                        });
                    }
                    tauri::WindowEvent::Focused(false) => {
                        if *just_shown.lock().unwrap() {
                            return;
                        }
                        let _ = window_for_event.hide();
                    }
                    _ => {}
                }
            });

            // Register default shortcut
            let state = app.state::<AppState>();
            let mods = state.shortcut_mods.lock().unwrap().clone();
            let key = state.shortcut_key.lock().unwrap().clone();
            drop(state);
            register_shortcut(app.handle(), mods, key);

            // Build tray icon
            let show_item = MenuItem::with_id(app, "show", "显示 ToDo", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("ToDo  (Ctrl + 2)")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_window_at(app),
                    "hide" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_window_at(tray.app_handle());
                    }
                })
                .build(app)?;

            if let Ok(mut tray_store) = app.state::<AppState>().tray.lock() {
                *tray_store = Some(tray);
            }

            // Initialize data directory for tasks
            if let Some(app_dir) = app.path().app_data_dir().ok() {
                let data_dir = app_dir.join(DATA_DIR);
                let _ = fs::create_dir_all(&data_dir);
                let state = app.state::<AppState>();
                *state.tasks.data_dir.lock().unwrap() = Some(data_dir);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
