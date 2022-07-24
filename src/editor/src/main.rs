use maths::rect2d::Rect2D;

use plateform::{Platform};
use plateform::window::{PlatformEvent, WindowCreateInfos, WindowFlagBits, WindowFlags};
use plateform_window::PlatformWin32;

fn main() {	
	// We use a win32 backend
	let platform = PlatformWin32::new();
	
	// Create main window	
	let _main_window = platform.create_window(WindowCreateInfos {
		name: "Engine - 0.1.0".to_string(),
		geometry: Rect2D::rect(300, 400, 800, 600),
		window_flags: WindowFlags::from_flag(WindowFlagBits::Resizable),
		background_alpha: 255
	}).unwrap();
	_main_window.lock().unwrap().show();

	'game_loop: loop {
		while let Some(message) = platform.poll_event() {
			match message {
				PlatformEvent::WindowClosed(_window) => {
					break 'game_loop;
				}
				PlatformEvent::WindowResized(_window, _width, _height) => {}
			}
		}
	}
}