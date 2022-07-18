use maths::rect2d::Rect2D;
use maths::vec2::{Vec2, Vec2F32, Vec2F64, Vec2i32};
use plateform::{Platform, WindowCreateInfos};
use plateform_window::PlatformWindows;

fn main() {	
	
	let test = Vec2F64::new(1.0, 2.0);
	let test2 = Vec2F64::new(1.0, 2.0);
	
	let test3 = test + test2 * test.y;
	
	println!("result = {}", test3.x);
	
	let platform = PlatformWindows::new();
	let main_window = platform.create_window(WindowCreateInfos {
		name: "Primary window".to_string(),
		geometry: Rect2D::new(0, 0, 200, 200),
		window_flags: Default::default()
	});
}