use std::ffi::c_void;
use std::ptr::NonNull;

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

pub trait GfxBuffer {
    fn submit_data(&self);
    fn resize_buffer(&self);
    fn get_data_buffer(&self) -> Option<NonNull<c_void>>;
}