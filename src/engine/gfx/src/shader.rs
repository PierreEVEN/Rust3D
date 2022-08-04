use std::collections::HashMap;
use std::sync::Arc;
use crate::GfxInterface;

pub type PassID = String;

pub struct Shader {
    permutations: HashMap<PassID, Arc<dyn ShaderPermutation>>,
    gfx: Arc<dyn GfxInterface>,
}

impl Shader {
    pub fn new(gfx: Arc<dyn GfxInterface>) -> Self {
        Self {
            permutations: HashMap::new(),
            gfx,
        }
    }

    pub fn update_pass(&mut self, pass: PassID, spirv: &Vec<u32>) {
        self.permutations.insert(pass, self.gfx.get_shader_backend().create_shader_permutation(spirv));
    }
    pub fn get_permutation(&self, pass: &PassID) -> Result<Arc<dyn ShaderPermutation>, ()> {
        match self.permutations.get(pass) {
            None => { Err(()) }
            Some(perm) => { Ok(perm.clone()) }
        }
    }
}

pub trait ShaderBackend {
    fn create_shader_permutation(&self, spirv: &Vec<u32>) -> Arc<dyn ShaderPermutation>;
}

pub trait ShaderPermutation {
    fn get_(&self) {}
    
    
}