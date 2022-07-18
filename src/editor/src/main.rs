use maths::rect2d::Rect2D;
use maths::vec2::{Vec2, Vec2F32, Vec2F64, Vec2i32};
use plateform::{Platform, WindowCreateInfos};
use plateform_window::PlatformWindows;

fn main() {	
	
	let platform = PlatformWindows::new();
	let main_window = platform.create_window(WindowCreateInfos {
		name: "Primary window".to_string(),
		geometry: Rect2D::new(0, 0, 200, 200),
		window_flags: Default::default()
	});
}