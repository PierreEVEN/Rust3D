use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

use ash::vk::{Buffer, BufferUsageFlags, DeviceSize};
use gpu_allocator::{AllocationError, MemoryLocation};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};

use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferMemory, BufferType, BufferUsage, GfxBuffer};
use gfx::GfxRef;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan};

pub struct VkBufferAccess(MemoryLocation);

impl Deref for VkBufferAccess {
    type Target = MemoryLocation;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BufferAccess> for VkBufferAccess {
    fn from(access: BufferAccess) -> Self {
        VkBufferAccess(
            match access {
                BufferAccess::Default => { MemoryLocation::CpuToGpu }
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
                BufferUsage::IndexData => { BufferUsageFlags::INDEX_BUFFER }
                BufferUsage::VertexData => { BufferUsageFlags::VERTEX_BUFFER }
                BufferUsage::GpuMemory => { BufferUsageFlags::STORAGE_BUFFER }
                BufferUsage::UniformBuffer => { BufferUsageFlags::UNIFORM_BUFFER }
                BufferUsage::IndirectDrawArgument => { BufferUsageFlags::INDIRECT_BUFFER }
                BufferUsage::TransferMemory => { BufferUsageFlags::TRANSFER_DST }
            })
    }
}

pub struct VkBuffer {
    pub allocation: Allocation,
    pub buffer: Buffer,
    allocator: Arc<RefCell<Allocator>>,
}

impl GfxBuffer for VkBuffer {
    fn resize_buffer(&self) {
        todo!()
    }

    fn get_buffer_memory(&self) -> BufferMemory {
        BufferMemory::from(match self.allocation.mapped_ptr() {
            None => { panic!("memory is not host visible") }
            Some(allocation) => { allocation }
        }.as_ptr())
    }

    fn submit_data(&self, _: &BufferMemory) {}
}

impl VkBuffer {
    pub fn new(gfx: &GfxRef, create_infos: &BufferCreateInfo) -> Self {
        let mut usage = VkBufferUsage::from(create_infos.usage).0;

        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();

        if create_infos.buffer_type != BufferType::Immutable
        {
            usage |= BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::TRANSFER_SRC;
        }

        let ci_buffer = ash::vk::BufferCreateInfo::builder()
            .size(create_infos.size as DeviceSize)
            .usage(usage);

        let buffer = unsafe { gfx_object!(*device).device.create_buffer(&ci_buffer, None) }.unwrap();
        let requirements = unsafe { gfx_object!(*device).device.get_buffer_memory_requirements(buffer) };
        
        let allocator = gfx_object!(*device).allocator.clone();

        let allocation = (&*allocator).borrow_mut().allocate(&AllocationCreateDesc {
            name: "buffer allocation",
            requirements,
            location: *VkBufferAccess::from(create_infos.access),
            linear: false,
        });

        Self {
            allocation: match allocation {
                Ok(alloc) => {
                    unsafe { gfx_object!(*device).device.bind_buffer_memory(buffer, alloc.memory(), alloc.offset()).unwrap() };
                    alloc
                }
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
            },
            buffer,
            allocator,
        }
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        (&*self.allocator).borrow_mut().free(std::mem::replace(&mut self.allocation, Allocation::default())).expect("failed to free buffer");
    }
}
