use tauri::{AppHandle, Manager, PhysicalPosition, Position, WebviewWindow};

/// 面板与屏幕右缘的间距（物理像素）
const EDGE_MARGIN: i32 = 12;
/// 面板与屏幕顶缘的间距（物理像素）：为 GNOME 顶栏预留
const TOP_MARGIN: i32 = 40;

/// 左键点击托盘图标：切换主面板显隐。
pub fn toggle_panel(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show_panel(app);
    }
}

/// 定位并显示主面板。
///
/// ADR-0002 普通窗口模式：不再吸附托盘图标（Linux AppIndicator 不上报图标
/// 几何信息）。X11 下定位到主显示器右上角；Wayland 合成器忽略 set_position，
/// 窗口位置由合成器决定（通常居中），属已接受的降级，勿"修复"回吸附式。
pub fn show_panel(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        tracing::warn!("show_panel: 找不到 main 窗口");
        return;
    };
    position_top_right(app, &window);
    let _ = window.show();
    let _ = window.set_focus();
}

/// 第二个实例启动 / 全局热键唤起时显示主面板（与普通显示路径一致）。
pub fn show_panel_for_second_instance(app: &AppHandle) {
    show_panel(app);
}

/// 把面板定位到主显示器右上角（X11 生效；Wayland 下为 no-op）。
fn position_top_right(app: &AppHandle, window: &WebviewWindow) {
    let Ok(Some(monitor)) = app.primary_monitor() else {
        return;
    };
    let size = window.outer_size().unwrap_or_default();
    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let x = mon_pos.x + mon_size.width as i32 - size.width as i32 - EDGE_MARGIN;
    let y = mon_pos.y + TOP_MARGIN;
    let _ = window.set_position(Position::Physical(PhysicalPosition::new(x, y)));
}
