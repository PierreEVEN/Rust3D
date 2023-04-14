use ecs::ecs::Ecs;
use gfx::command_buffer::GfxCommandBuffer;

pub struct Scene {
    ecs: Ecs,
    draw_system: Vec<u8>,
    tick_systems: Vec<u8>,
}

impl Scene {
    pub fn collect_render_commands(&self, command_buffer: &dyn GfxCommandBuffer, render_flags : u32) {
        // run parallel
        
        
    }
    
    pub fn tick(&self) {
        // run parallel for non mutable systems
        // then run sequential for mutable system
    }
}

fn _test() {
    //let scene: Scene;
    
    loop {
        //scene.tick();
        
        let flags = [0, 1, 2, 3];
        
        for _i in flags { // parallel
            /*
            let cmd: dyn CommandBuffer = new();
            scene.collect_render_commands(&cmd, _i);
            cmd.submit();
             */
        }
    }   
}


