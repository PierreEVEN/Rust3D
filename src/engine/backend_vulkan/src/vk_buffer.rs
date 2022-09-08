use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::NonNull;
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

pub struct RbBuffer {
    create_infos: BufferCreateInfo,
}

impl GfxImageBuilder<BufferContainer> for RbBuffer {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> BufferContainer {
        let buffer_size = if self.create_infos.size <= 0 { 1 } else { self.create_infos.size };
        let mut usage = VkBufferUsage::from(self.create_infos.usage).0;

        if self.create_infos.buffer_type != BufferType::Immutable
        {
            usage |= BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::TRANSFER_SRC;
        }

        let ci_buffer = ash::vk::BufferCreateInfo::builder()
            .size(buffer_size as DeviceSize)
            .usage(usage)
            .build();

        let buffer = vk_check!(unsafe { gfx.cast::<GfxVulkan>().device.handle.create_buffer(&ci_buffer, None) });
        let requirements = unsafe { gfx.cast::<GfxVulkan>().device.handle.get_buffer_memory_requirements(buffer) };

        let allocation = match gfx.cast::<GfxVulkan>().device.allocator.write().unwrap().allocate(&AllocationCreateDesc {
            name: "buffer allocation",
            requirements,
            location: *VkBufferAccess::from(self.create_infos.access),
            linear: false,
        }) {
            Ok(allocation) => { allocation }
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
        };

        unsafe { gfx.cast::<GfxVulkan>().device.handle.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()).unwrap() };

        BufferContainer {
            buffer,
            allocation: Arc::new(RwLock::new(allocation)),
            gfx: gfx.clone(),
        }
    }
}

#[derive(Clone)]
struct BufferContainer {
    buffer: Buffer,
    allocation: Arc<RwLock<Allocation>>,
    gfx: GfxRef,
}

impl Drop for BufferContainer {
    fn drop(&mut self) {
        self.gfx.cast::<GfxVulkan>().device.allocator.write().unwrap().free(std::mem::replace(&mut self.allocation.write().unwrap(), Allocation::default())).expect("failed to free buffer");
    }
}

pub struct VkBuffer {
    container: GfxResource<BufferContainer>,
    buffer_size: AtomicU32,
    gfx: GfxRef,
    create_infos: BufferCreateInfo,
}

impl GfxBuffer for VkBuffer {
    fn set_data(&self, frame: &GfxImageID, start_offset: u32, data: &[u8]) {
        if self.create_infos.buffer_type == BufferType::Immutable {
            panic!("Modifying data on immutable buffers is not allowed");
        }
        self.resize_buffer(start_offset + data.len() as u32);

        match self.create_infos.buffer_type {
            BufferType::Immutable => {
                panic!("Modifying data on immutable buffers is not allowed");
            }
            BufferType::Static => {
                match self.container.get_static().allocation.read().unwrap().mapped_ptr() {
                    None => { panic!("memory is not host visible") }
                    Some(allocation) => unsafe {
                        data.as_ptr().copy_to((allocation.as_ptr() as *mut u8).offset(start_offset as isize), data.len());
                    }
                }
            }
            BufferType::Dynamic => {
                todo!()
            }
            BufferType::Immediate => {
                match self.container.get(frame).allocation.read().unwrap().mapped_ptr() {
                    None => { panic!("memory is not host visible") }
                    Some(allocation_ptr) => unsafe {
                        data.as_ptr().copy_to((allocation_ptr.as_ptr() as *mut u8).offset(start_offset as isize), data.len());
                    }
                }
            }
        }
    }

    fn resize_buffer(&self, new_size: u32) {
        if self.buffer_size.load(Ordering::Acquire) == new_size { return; }
        self.buffer_size.store(new_size, Ordering::Release);

        match self.create_infos.buffer_type {
            BufferType::Immutable => {
                panic!("an immutable buffer is not resizable");
            }
            BufferType::Static => {
                vk_check!(unsafe { self.gfx.cast::<GfxVulkan>().device.handle.device_wait_idle() });
                self.container.invalidate(&self.gfx, RbBuffer { create_infos: self.create_infos });
            }
            BufferType::Dynamic => {
                todo!();
            }
            BufferType::Immediate => {
                self.container.invalidate(&self.gfx, RbBuffer { create_infos: self.create_infos });
            }
        }
    }

    fn buffer_size(&self) -> u32 {
        self.buffer_size.load(Ordering::Acquire)
    }

    fn create_infos(&self) -> &BufferCreateInfo {
        &self.create_infos
    }
}

impl VkBuffer {
    pub fn new(gfx: &GfxRef, create_infos: &BufferCreateInfo) -> Self {
        let allocation = match create_infos.buffer_type {
            BufferType::Immutable | BufferType::Static => {
                GfxResource::new_static(gfx, RbBuffer { create_infos: *create_infos })
            }
            _ => { GfxResource::new(gfx, RbBuffer { create_infos: *create_infos }) }
        };

        Self {
            container: allocation,
            buffer_size: AtomicU32::new(create_infos.size),
            gfx: gfx.clone(),
            create_infos: *create_infos,
        }
    }

    pub fn get_handle(&self, image: &GfxImageID) -> Buffer {
        match self.create_infos.buffer_type {
            BufferType::Immutable | BufferType::Static => { self.container.get_static().buffer }
            BufferType::Dynamic | BufferType::Immediate => { self.container.get(image).buffer }
        }
    }
}