use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use shader_base::{BindPoint, ShaderInterface};
use shader_base::pass_id::PassID;
use crate::command_buffer::GfxCommandBuffer;
use crate::Gfx;
use crate::image::GfxImage;
use crate::image_sampler::ImageSampler;
use crate::shader::ShaderProgram;
use crate::shader_instance::ShaderInstance;

#[derive(Default, Clone)]
pub struct MaterialBindings {
    images: HashMap<BindPoint, Arc<dyn GfxImage>>,
    samplers: HashMap<BindPoint, Arc<dyn ImageSampler>>,
}

pub struct PassMaterialData {
    bindings_dirty: AtomicBool,
    instance: Arc<dyn ShaderInstance>,
    master: Arc<dyn ShaderProgram>,
}

impl Clone for PassMaterialData {
    fn clone(&self) -> Self {
        Self {
            bindings_dirty: AtomicBool::new(true),
            instance: self.master.instantiate(),
            master: self.master.clone(),
        }
    }
}

#[derive(Default)]
pub struct Material {
    passes: RwLock<HashMap<PassID, Option<PassMaterialData>>>,
    shader_interface: RwLock<Option<Arc<dyn ShaderInterface>>>,
    bindings: Arc<RwLock<MaterialBindings>>,
}

impl Clone for Material {
    fn clone(&self) -> Self {
        Self {
            passes: RwLock::new((&*self.passes.read().unwrap()).clone()),
            shader_interface: RwLock::new(self.shader_interface.read().unwrap().clone()),
            bindings: Arc::new(RwLock::new((*self.bindings.read().unwrap()).clone())),
        }
    }
}

impl Material {
    pub fn set_shader<T: 'static + ShaderInterface>(&self, shader: T) {
        if shader.get_errors().len() > 0 {
            for error in shader.get_errors() {
                logger::error!("{:?}", error);
            }
            return;
        }
        *self.shader_interface.write().unwrap() = Some(Arc::new(shader));
    }

    pub fn bind_to(&self, command_buffer: &dyn GfxCommandBuffer) {
        let pass = command_buffer.get_pass_id();
        if let Some(program) = self.get_program(&pass) {
            command_buffer.bind_program(&program);
            if let Some(instance) = self.get_instance(&pass) {
                command_buffer.bind_shader_instance(&instance);
            }
        }
    }

    pub fn bind_texture(&self, bind_point: BindPoint, texture: Arc<dyn GfxImage>) {
        let bindings = &mut *self.bindings.write().unwrap();
        bindings.images.insert(bind_point, texture);
        self.mark_bindings_dirty();
    }

    pub fn bind_sampler(&self, bind_point: BindPoint, sampler: Arc<dyn ImageSampler>) {
        let bindings = &mut *self.bindings.write().unwrap();
        bindings.samplers.insert(bind_point, sampler);
        self.mark_bindings_dirty();
    }

    fn mark_bindings_dirty(&self) {
        for pass in self.passes.read().unwrap().values() {
            if let Some(pass_data) = pass {
                pass_data.bindings_dirty.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn get_program(&self, pass_id: &PassID) -> Option<Arc<dyn ShaderProgram>> {
        // If program was previously loaded
        if let Some(program) = self.passes.read().unwrap().get(pass_id) {
            return match program {
                None => { None }
                Some(mat_data) => { Some(mat_data.master.clone()) }
            };
        }

        // Else load it
        match self.shader_interface.read().unwrap().as_ref() {
            None => {None}
            Some(shi) => {
                return match Gfx::get().get_program_pool().find_or_create_program(pass_id, shi) {
                    None => {
                        self.passes.write().unwrap().insert(pass_id.clone(), None);
                        None
                    }
                    Some(program) => {
                        self.passes.write().unwrap().insert(pass_id.clone(), Some(PassMaterialData {
                            bindings_dirty: AtomicBool::new(false),
                            instance: program.instantiate(),
                            master: program.clone(),
                        }));
                        Some(program)
                    }
                };
            }
        }
    }

    pub fn get_instance(&self, pass_id: &PassID) -> Option<Arc<dyn ShaderInstance>> {
        if let Some(pass) = self.passes.read().unwrap().get(pass_id) {
            return match pass {
                None => { None }
                Some(data) => { Some(data.instance.clone()) }
            }
        }

        // Try refresh program
        match self.get_program(&pass_id) {
            None => { None }
            Some(_) => { self.get_instance(pass_id) } // Try again
        }
    }
}
