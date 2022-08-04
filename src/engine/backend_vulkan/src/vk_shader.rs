use std::mem;
use std::ptr::null;
use std::sync::Arc;
use ash::vk::{ShaderModule, ShaderModuleCreateFlags, ShaderModuleCreateInfo, StructureType};
use gfx::buffer::BufferAccess::Default;
use gfx::GfxInterface;
use gfx::shader::{ShaderBackend, ShaderPermutation};
use crate::{gfx_object, GfxVulkan, vk_check};

pub struct VkShaderBackend {
    gfx: Arc<GfxVulkan>,
}

impl VkShaderBackend {
    pub fn new(gfx: Arc<GfxVulkan>) -> Self {
        Self {
            gfx
        }
    }
}

impl ShaderBackend for VkShaderBackend {
    fn create_shader_permutation(&self, spirv: &Vec<u32>) -> Arc<dyn ShaderPermutation> {
        Arc::new(VkShaderPermutation::new(self.gfx.as_ref() as &GfxVulkan, spirv))
    }
}

pub struct VkShaderPermutation {
    shader_module: ShaderModule
    
}

impl VkShaderPermutation {
    pub fn new(gfx: &GfxVulkan, spirv: &Vec<u32>) -> Self {
        let ci_shader_module = ShaderModuleCreateInfo {
            s_type: StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: spirv.len() * mem::size_of::<u32>(),
            p_code: spirv.as_ptr(),
            flags: ShaderModuleCreateFlags::default(),
            p_next: null(),
        };

        let shader_module = vk_check!(unsafe { gfx_object!(gfx.device).device.create_shader_module(&ci_shader_module, None) });

        Self {
            shader_module
        }
    }
}

impl ShaderPermutation for VkShaderPermutation {
    fn get_(&self) {
        todo!()
    }
}