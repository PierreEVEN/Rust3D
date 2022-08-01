use ash::vk::{BufferUsageFlags, DeviceSize, MemoryRequirements};
use gpu_allocator::{AllocationError, MemoryLocation};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};

use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferUsage, GfxBuffer};

use crate::{gfx_object, GfxVulkan};

pub struct VkBufferAccess(MemoryLocation);
impl From<BufferAccess> for VkBufferAccess {
    fn from(access: BufferAccess) -> Self {
        VkBufferAccess(
            match access {
                BufferAccess::Default => { MemoryLocation::Unknown }
                BufferAccess::GpuOnly => { MemoryLocation::GpuOnly }
                BufferAccess::CpuToGpu => { MemoryLocation::CpuToGpu }
                BufferAccess::GpuToCpu => { MemoryLocation::GpuToCpu }
            })
    }
}
pub struct VkBufferUsage(BufferUsageFlags);
impl From<BufferUsage> for VkBufferUsage {
    fn from(usage: BufferUsage) -> Self {
        VkBufferUsage(
            match usage {
                BufferUsage::IndexData => {BufferUsageFlags::INDEX_BUFFER}
                BufferUsage::VertexData => {BufferUsageFlags::VERTEX_BUFFER}
                BufferUsage::GpuMemory => {BufferUsageFlags::STORAGE_BUFFER}
                BufferUsage::UniformBuffer => {BufferUsageFlags::UNIFORM_BUFFER}
                BufferUsage::IndirectDrawArgument => {BufferUsageFlags::INDIRECT_BUFFER}
                BufferUsage::TransferMemory => {BufferUsageFlags::TRANSFER_DST}
            })
    }
}

pub struct VkBuffer {
    pub buffer: Option<Allocation>,
}

impl GfxBuffer for VkBuffer {}


impl VkBuffer {
    pub fn new(gfx: &GfxVulkan, create_infos: &BufferCreateInfo) -> Self {

        let ci_buffer = ash::vk::BufferCreateInfo::builder()
            .size(create_infos.size as DeviceSize)
            .usage(VkBufferUsage::from(create_infos.usage).0);


        let buffer = unsafe { gfx_object!(gfx.device).device.create_buffer(&ci_buffer, None) }.unwrap();
        let requirements = unsafe { gfx_object!(gfx.device).device.get_buffer_memory_requirements(buffer) };
        
        let mut allocator = gfx_object!(gfx.device).allocator.lock().expect("test");
        
        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: "buffer allocation",
            requirements,
            location: VkBufferAccess::from(create_infos.access.clone()).0,
            linear: false,
        });

        Self {
            buffer:
            match allocation {
                Ok(alloc) => { Some(alloc) }
                Err(alloc_error) => {
                    match alloc_error {
                        AllocationError::OutOfMemory => { panic!("failed to create buffer : out of memory") }
                        AllocationError::FailedToMap(_string) => { panic!("failed to create buffer : failed to map : {_string}") }
                        AllocationError::NoCompatibleMemoryTypeFound => { panic!("failed to create buffer : no compatible memory type found") }
                        AllocationError::InvalidAllocationCreateDesc => { panic!("failed to create buffer : invalid buffer create infos") }
                        AllocationError::InvalidAllocatorCreateDesc(_string) => { panic!("failed to create buffer : invalid allocator create infos : {_string}") }
                        AllocationError::Internal(_string) => { panic!("failed to create buffer : {_string}") }
                    }
                }
            }
        }
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
    }
}
