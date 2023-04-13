use crate::GfxCast;
use crate::surface::GfxImageID;

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

#[derive(Copy, Clone)]
pub struct BufferCreateInfo {
    pub buffer_type: BufferType,
    pub usage: BufferUsage,
    pub access: BufferAccess,
    pub size: u32,
}

pub struct BufferMemory {
    size: usize,
    data: *const u8,
}

impl BufferMemory {
    pub fn from(data: *const u8, size: usize) -> Self {
        Self { data, size }
    }

    pub fn from_struct<T: Sized>(structure: &T) -> Self {
        Self {
            data: structure as *const T as *const u8,
            size: ::std::mem::size_of::<T>(),
        }
    }

    pub fn get_ptr(&self, offset: usize) -> *mut u8 {
        let data = self.data as *mut u8;
        unsafe { data.add(offset) }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { ::std::slice::from_raw_parts(self.data, self.size) }
    }
}

pub trait GfxBuffer: GfxCast {
    fn set_data(&self, frame: &GfxImageID, start_offset: u32, data: &[u8]);
    fn resize_buffer(&self, new_size: u32);
    fn buffer_size(&self) -> u32;
    fn create_infos(&self) -> &BufferCreateInfo;
}

impl dyn GfxBuffer {
    pub fn cast<U: GfxBuffer + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}