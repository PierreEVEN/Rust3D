use core::panicking::AssertKind::Ne;
use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

use ash::vk::{Buffer, BufferUsageFlags, DeviceSize};
use gpu_allocator::{AllocationError, MemoryLocation};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};

use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferMemory, BufferType, BufferUsage, GfxBuffer};
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::surface::GfxImageID;

use crate::{GfxVulkan, vk_check};

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

pub struct RbBuffer {}

impl GfxImageBuilder<Arc<RwLock<Allocation>>> for RbBuffer {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> Arc<RwLock<Allocation>> {
        todo!()
    }
}

pub struct VkBuffer {
    pub allocation: GfxResource<Arc<RwLock<Allocation>>>,
    pub handle: Buffer,
    buffer_size: AtomicU32,
    buffer_type: BufferType,
    gfx: GfxRef,
}

impl GfxBuffer for VkBuffer {
    fn set_data(&self, start_offset: u32, data: &[u8]) {
        if self.buffer_type == BufferType::Immutable {
            panic!("Modifying data on immutable buffers is not allowed");
        }
        self.resize_buffer(start_offset + data.len() as u32);
        
        match self.buffer_type {
            BufferType::Immutable => {}
            BufferType::Static => {
                match self.allocation.get_static().write().unwrap().mapped_ptr() {
                    None => { panic!("memory is not host visible") }
                    Some(allocation) => unsafe {
                        data.as_ptr().copy_to((allocation.as_ptr() as *mut u8).offset(start_offset as isize), data.len());
                    }
                }
            }
            BufferType::Dynamic => {
                
            }
            BufferType::Immediate => {
                
            }
        }
    }

    fn resize_buffer(&self, new_size: u32) {
        if self.buffer_size.load(Ordering::Acquire) == new_size { return; }
        self.buffer_size.store(new_size, Ordering::Release);

        match self.buffer_type {
            BufferType::Immutable => {
                panic!("an immutable buffer is not resizable");
            }
            BufferType::Static => {
                vk_check!(unsafe { self.gfx.cast::<GfxVulkan>().device.handle.device_wait_idle() });
                self.allocation.invalidate(&self.gfx, RbBuffer {});
            }
            BufferType::Dynamic => {
                todo!();
            }
            BufferType::Immediate => {
                self.allocation.invalidate(&self.gfx, RbBuffer {});
            }
        }
    }

    fn get_buffer_memory(&self) -> BufferMemory {
        match self.buffer_type {
            BufferType::Immutable | BufferType::Static => {}
            BufferType::Dynamic | BufferType::Immediate => {}
        }

        BufferMemory::from(match self.allocation.mapped_ptr() {
            None => { panic!("memory is not host visible") }
            Some(allocation) => {
                allocation.as_ptr() as *const u8
            }
        }, self.allocation.size() as usize)
    }

    fn buffer_size(&self) -> u32 {
        self.allocation.size() as u32
    }
}

impl VkBuffer {
    pub fn new(gfx: &GfxRef, create_infos: &BufferCreateInfo) -> Self {
        let buffer_size = if create_infos.size <= 0 { 1 } else { create_infos.size };
        let mut usage = VkBufferUsage::from(create_infos.usage).0;

        if create_infos.buffer_type != BufferType::Immutable
        {
            usage |= BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::TRANSFER_SRC;
        }

        let ci_buffer = ash::vk::BufferCreateInfo::builder()
            .size(buffer_size as DeviceSize)
            .usage(usage);

        let buffer = unsafe { gfx.cast::<GfxVulkan>().device.handle.create_buffer(&ci_buffer, None) }.unwrap();
        let requirements = unsafe { gfx.cast::<GfxVulkan>().device.handle.get_buffer_memory_requirements(buffer) };

        let allocator = gfx.cast::<GfxVulkan>().device.allocator.clone();

        let allocation = (&*allocator).borrow_mut().allocate(&AllocationCreateDesc {
            name: "buffer allocation",
            requirements,
            location: *VkBufferAccess::from(create_infos.access),
            linear: false,
        });

        Self {
            allocation: match allocation {
                Ok(alloc) => {
                    unsafe { gfx.cast::<GfxVulkan>().device.handle.bind_buffer_memory(buffer, alloc.memory(), alloc.offset()).unwrap() };
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
            handle: buffer,
            allocator,
        }
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        (&*self.allocator).borrow_mut().free(std::mem::replace(&mut self.allocation, Allocation::default())).expect("failed to free buffer");
    }
}
