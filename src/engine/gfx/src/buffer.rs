use std::ffi::c_void;
use crate::GfxCast;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BufferType
{
    // No allowed updates
    Immutable,
    // Pretty never updated. Updating data would cause some freezes
    Static,
    // Data is stored internally, then automatically submitted. Can lead to a memory overhead depending on the get size.
    Dynamic,
    // Data need to be submitted every frames
    Immediate,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BufferUsage {
    // used as index get
    IndexData,
    // used as vertex get
    VertexData,
    // used as storage get
    GpuMemory,
    // used as uniform get
    UniformBuffer,
    // used for indirect draw commands
    IndirectDrawArgument,
    // used for indirect draw commands
    TransferMemory,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BufferAccess
{
    // Choose best configuration
    Default,
    // Data will be cached on GPU
    GpuOnly,
    // frequent transfer from CPU to GPU
    CpuToGpu,
    // frequent transfer from GPU to CPU
    GpuToCpu,
}

pub struct BufferCreateInfo {
    pub buffer_type: BufferType,
    pub usage: BufferUsage,
    pub access: BufferAccess,
    pub size: u32,
    pub alignment: u32,
    pub memory_type_bits: u32,
}

pub struct BufferMemory {
    data: *const c_void,
}

impl BufferMemory {
    pub fn from(data: *const c_void) -> Self {
        Self { data }
    }
    pub fn get_ptr(&self, offset: usize) -> *mut u8 {
        let data = self.data as *mut u8;
        unsafe { data.offset(offset as isize) }
    }
}

pub trait GfxBuffer : GfxCast {
    fn resize_buffer(&self);
    fn get_buffer_memory(&self) -> BufferMemory;
    fn submit_data(&self, memory: &BufferMemory);
}