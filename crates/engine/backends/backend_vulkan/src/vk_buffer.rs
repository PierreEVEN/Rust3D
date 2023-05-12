use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use ash::vk;
use ash::vk::Handle;
use gpu_allocator::vulkan;
use gpu_allocator::{AllocationError, MemoryLocation};

use gfx::buffer::{BufferAccess, BufferCreateInfo, BufferType, BufferUsage, GfxBuffer};
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::surface::GfxImageID;

use crate::{vk_check, GfxVulkan};

pub struct VkBufferAccess(MemoryLocation);

impl Deref for VkBufferAccess {
    type Target = MemoryLocation;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BufferAccess> for VkBufferAccess {
    fn from(access: BufferAccess) -> Self {
        VkBufferAccess(match access {
            BufferAccess::Default => MemoryLocation::CpuToGpu,
            BufferAccess::GpuOnly => MemoryLocation::GpuOnly,
            BufferAccess::CpuToGpu => MemoryLocation::CpuToGpu,
            BufferAccess::GpuToCpu => MemoryLocation::GpuToCpu,
        })
    }
}

pub struct VkBufferUsage(vk::BufferUsageFlags);

impl From<BufferUsage> for VkBufferUsage {
    fn from(usage: BufferUsage) -> Self {
        VkBufferUsage(match usage {
            BufferUsage::IndexData => vk::BufferUsageFlags::INDEX_BUFFER,
            BufferUsage::VertexData => vk::BufferUsageFlags::VERTEX_BUFFER,
            BufferUsage::GpuMemory => vk::BufferUsageFlags::STORAGE_BUFFER,
            BufferUsage::UniformBuffer => vk::BufferUsageFlags::UNIFORM_BUFFER,
            BufferUsage::IndirectDrawArgument => vk::BufferUsageFlags::INDIRECT_BUFFER,
            BufferUsage::TransferMemory => vk::BufferUsageFlags::TRANSFER_DST,
        })
    }
}

pub struct RbBuffer {
    create_infos: BufferCreateInfo,
    size_override: u32,
    name: String,
}

impl GfxImageBuilder<Arc<BufferContainer>> for RbBuffer {
    fn build(&self, image_id: &GfxImageID) -> Arc<BufferContainer> {
        let buffer_size = if self.size_override == 0 {
            1
        } else {
            self.size_override
        };
        let mut usage = VkBufferUsage::from(self.create_infos.usage).0;

        if self.create_infos.buffer_type != BufferType::Immutable {
            usage |= vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC;
        }

        let ci_buffer = vk::BufferCreateInfo::builder()
            .size(buffer_size as vk::DeviceSize)
            .usage(usage)
            .build();

        let buffer = GfxVulkan::get().set_vk_object_name(
            vk_check!(unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .create_buffer(&ci_buffer, None)
            }),
            format!("buffer handle\t\t: {}@{}", self.name, image_id).as_str(),
        );
        let requirements = unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .get_buffer_memory_requirements(buffer)
        };

        let allocation = match unsafe { GfxVulkan::get().device.assume_init_ref() }
            .allocator
            .write()
            .unwrap()
            .allocate(&vulkan::AllocationCreateDesc {
                name: "buffer allocation",
                requirements,
                location: *VkBufferAccess::from(self.create_infos.access),
                linear: false,
                allocation_scheme: vulkan::AllocationScheme::DedicatedBuffer(buffer),
            }) {
            Ok(allocation) => allocation,
            Err(alloc_error) => match alloc_error {
                AllocationError::OutOfMemory => {
                    logger::fatal!("failed to create buffer : out of memory")
                }
                AllocationError::FailedToMap(_string) => {
                    logger::fatal!("failed to create buffer : failed to map : {_string}")
                }
                AllocationError::NoCompatibleMemoryTypeFound => {
                    logger::fatal!("failed to create buffer : no compatible memory type found")
                }
                AllocationError::InvalidAllocationCreateDesc => {
                    logger::fatal!("failed to create buffer : invalid buffer create infos")
                }
                AllocationError::InvalidAllocatorCreateDesc(_string) => {
                    logger::fatal!(
                        "failed to create buffer : invalid allocator create infos : {_string}"
                    )
                }
                AllocationError::Internal(_string) => {
                    logger::fatal!("failed to create buffer : {_string}")
                }
            },
        };

        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .bind_buffer_memory(
                    buffer,
                    GfxVulkan::get().set_vk_object_name(
                        allocation.memory(),
                        format!("buffer memory\t\t: {}@{}", self.name, image_id).as_str(),
                    ),
                    allocation.offset(),
                )
                .unwrap()
        };

        match allocation.mapped_ptr() {
            None => {
                logger::fatal!("memory is not host visible")
            }
            Some(_) => {}
        }

        Arc::new(BufferContainer {
            buffer,
            allocation: RwLock::new(allocation),
        })
    }
}

struct BufferContainer {
    buffer: vk::Buffer,
    allocation: RwLock<vulkan::Allocation>,
}

impl Drop for BufferContainer {
    fn drop(&mut self) {
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .allocator
                .write()
                .unwrap()
                .free(std::mem::take(&mut self.allocation.write().unwrap()))
                .expect("failed to free buffer");
        }
    }
}

pub struct VkBuffer {
    container: GfxResource<Arc<BufferContainer>>,
    buffer_size: AtomicU32,
    create_infos: BufferCreateInfo,
    name: String,
}

impl GfxBuffer for VkBuffer {
    fn set_data(&self, frame: &GfxImageID, start_offset: u32, data: &[u8]) {
        if self.create_infos.buffer_type == BufferType::Immutable {
            logger::fatal!("Modifying data on immutable buffers is not allowed");
        }
        if start_offset + data.len() as u32 > self.buffer_size.load(Ordering::Acquire) {
            logger::fatal!(
                "buffer is to small : size={}, expected={}",
                self.buffer_size.load(Ordering::Acquire),
                start_offset + data.len() as u32
            );
        }

        unsafe {
            match self.create_infos.buffer_type {
                BufferType::Immutable => {
                    logger::fatal!("Modifying data on immutable buffers is not allowed");
                }
                BufferType::Static => {
                    match self
                        .container
                        .get_static()
                        .allocation
                        .read()
                        .unwrap()
                        .mapped_ptr()
                    {
                        None => {
                            logger::fatal!("memory is not host visible")
                        }
                        Some(allocation) => {
                            data.as_ptr().copy_to(
                                (allocation.as_ptr() as *mut u8).offset(start_offset as isize),
                                data.len(),
                            );
                        }
                    }
                }
                BufferType::Dynamic => {
                    todo!()
                }
                BufferType::Immediate => {
                    match self.container.get(frame).allocation.read() {
                        Ok(allocation) => match allocation.mapped_ptr() {
                            None => {
                                logger::fatal!(
                                    "memory [{}] is not host visible for frame {}",
                                    allocation.memory().as_raw(),
                                    frame.image_id()
                                );
                            }
                            Some(allocation_ptr) => {
                                data.as_ptr().copy_to(
                                    (allocation_ptr.as_ptr() as *mut u8)
                                        .offset(start_offset as isize),
                                    data.len(),
                                );
                            }
                        },
                        Err(_) => {
                            logger::fatal!("failed to read allocation")
                        }
                    };
                }
            }
        }
    }

    fn resize_buffer(&self, new_size: u32) {
        if self.buffer_size.load(Ordering::Acquire) == new_size {
            return;
        }
        self.buffer_size.store(new_size, Ordering::Release);

        match self.create_infos.buffer_type {
            BufferType::Immutable => {
                logger::fatal!("an immutable buffer is not resizable");
            }
            BufferType::Static => {
                vk_check!(unsafe {
                    GfxVulkan::get()
                        .device
                        .assume_init_ref()
                        .handle
                        .device_wait_idle()
                });
                self.container.invalidate(RbBuffer {
                    create_infos: self.create_infos,
                    size_override: new_size,
                    name: self.name.clone(),
                });
            }
            BufferType::Dynamic => {
                todo!();
            }
            BufferType::Immediate => {
                self.container.invalidate(RbBuffer {
                    create_infos: self.create_infos,
                    size_override: new_size,
                    name: self.name.clone(),
                });
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
    pub fn new(name: String, create_infos: &BufferCreateInfo) -> Self {
        let allocation = match create_infos.buffer_type {
            BufferType::Immutable | BufferType::Static => GfxResource::new_static(RbBuffer {
                create_infos: *create_infos,
                size_override: create_infos.size,
                name: name.clone(),
            }),
            _ => GfxResource::new(RbBuffer {
                create_infos: *create_infos,
                size_override: create_infos.size,
                name: name.clone(),
            }),
        };

        Self {
            container: allocation,
            buffer_size: AtomicU32::new(create_infos.size),
            create_infos: *create_infos,
            name,
        }
    }

    pub fn get_handle(&self, image: &GfxImageID) -> vk::Buffer {
        match self.create_infos.buffer_type {
            BufferType::Immutable | BufferType::Static => self.container.get_static().buffer,
            BufferType::Dynamic | BufferType::Immediate => self.container.get(image).buffer,
        }
    }
}
