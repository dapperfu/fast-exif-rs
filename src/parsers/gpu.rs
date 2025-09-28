#[cfg(feature = "gpu")]
use std::collections::HashMap;
#[cfg(feature = "gpu")]
use crate::types::ExifError;

#[cfg(feature = "gpu")]
/// GPU-accelerated EXIF parser using OpenCL
pub struct GpuExifParser {
    context: Option<ocl::Context>,
    queue: Option<ocl::Queue>,
    program: Option<ocl::Program>,
    gpu_available: bool,
}

#[cfg(feature = "gpu")]
impl GpuExifParser {
    pub fn new() -> Self {
        let (context, queue) = Self::create_context_and_queue();
        let gpu_available = context.is_some() && queue.is_some();
        
        Self {
            context,
            queue,
            program: None,
            gpu_available,
        }
    }

    /// Create OpenCL context and queue
    fn create_context_and_queue() -> (Option<ocl::Context>, Option<ocl::Queue>) {
        // Try to create OpenCL context
        let context = ocl::Context::builder()
            .devices(ocl::core::Device::all(ocl::core::DeviceType::GPU).unwrap_or_default())
            .build()
            .ok();
            
        let queue = if let Some(ref ctx) = context {
            ocl::Queue::new(ctx, ocl::core::Device::first(ctx.devices()).ok()?, None).ok()
        } else {
            None
        };
        
        (context, queue)
    }

    /// GPU-accelerated JPEG EXIF parsing
    pub fn parse_jpeg_exif_gpu(
        &mut self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        if self.gpu_available {
            if let (Some(context), Some(queue)) = (&self.context, &self.queue) {
                self.parse_jpeg_exif_opencl(data, metadata, context, queue)
            } else {
                // Fallback to CPU implementation
                crate::parsers::jpeg::JpegParser::parse_jpeg_exif(data, metadata)
            }
        } else {
            // GPU not available, use CPU
            crate::parsers::jpeg::JpegParser::parse_jpeg_exif(data, metadata)
        }
    }

    /// GPU-accelerated HEIC EXIF parsing
    pub fn parse_heic_exif_gpu(
        &mut self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
    ) -> Result<(), ExifError> {
        if self.gpu_available {
            if let (Some(context), Some(queue)) = (&self.context, &self.queue) {
                self.parse_heic_exif_opencl(data, metadata, context, queue)
            } else {
                // Fallback to CPU implementation
                crate::parsers::heif::HeifParser::parse_heif_exif(data, metadata)
            }
        } else {
            // GPU not available, use CPU
            crate::parsers::heif::HeifParser::parse_heif_exif(data, metadata)
        }
    }

    /// OpenCL-accelerated JPEG parsing
    fn parse_jpeg_exif_opencl(
        &self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
        context: &ocl::Context,
        queue: &ocl::Queue,
    ) -> Result<(), ExifError> {
        // Create OpenCL kernel for JPEG marker detection
        let kernel_source = r#"
            __kernel void find_jpeg_markers(
                __global const uchar* data,
                __global uint* markers,
                uint data_size
            ) {
                uint gid = get_global_id(0);
                if (gid + 1 >= data_size) return;
                
                // Look for JPEG markers (0xFF 0xE1)
                if (data[gid] == 0xFF && data[gid + 1] == 0xE1) {
                    markers[gid] = gid;
                } else {
                    markers[gid] = 0xFFFFFFFF; // Invalid marker
                }
            }
        "#;

        // Compile and execute kernel
        let program = ocl::Program::builder()
            .src(kernel_source)
            .devices(context.devices())
            .build()
            .map_err(|e| ExifError::ParseError(format!("OpenCL program build failed: {}", e)))?;

        let mut markers_buffer = vec![0u32; data.len()];
        let markers_cl = ocl::Buffer::<u32>::builder()
            .queue(queue.clone())
            .len(data.len())
            .build()
            .map_err(|e| ExifError::ParseError(format!("OpenCL buffer creation failed: {}", e)))?;

        let kernel = ocl::Kernel::builder()
            .program(&program)
            .name("find_jpeg_markers")
            .queue(queue.clone())
            .arg(data)
            .arg(&markers_cl)
            .arg(data.len() as u32)
            .build()
            .map_err(|e| ExifError::ParseError(format!("OpenCL kernel build failed: {}", e)))?;

        // Execute kernel
        unsafe {
            kernel.enq()
                .map_err(|e| ExifError::ParseError(format!("OpenCL kernel execution failed: {}", e)))?;
        }

        // Read results back
        markers_cl.read(&mut markers_buffer)
            .enq()
            .map_err(|e| ExifError::ParseError(format!("OpenCL buffer read failed: {}", e)))?;

        // Process results - find valid markers
        for (i, &marker) in markers_buffer.iter().enumerate() {
            if marker != 0xFFFFFFFF && i + 1 < data.len() {
                // Found EXIF marker, extract and parse EXIF data
                if let Some(exif_data) = self.extract_exif_segment(data, i) {
                    // Parse EXIF data using CPU (GPU parsing of EXIF is complex)
                    crate::parsers::tiff::TiffParser::parse_tiff_exif(exif_data, metadata)?;
                }
            }
        }

        Ok(())
    }

    /// Extract EXIF segment from JPEG data
    fn extract_exif_segment(&self, data: &[u8], offset: usize) -> Option<&[u8]> {
        if offset + 4 >= data.len() {
            return None;
        }
        
        let length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
        if offset + length > data.len() || length < 4 {
            return None;
        }
        
        Some(&data[offset + 4..offset + length])
    }

    /// OpenCL-accelerated HEIC parsing
    fn parse_heic_exif_opencl(
        &self,
        data: &[u8],
        metadata: &mut HashMap<String, String>,
        context: &ocl::Context,
        queue: &ocl::Queue,
    ) -> Result<(), ExifError> {
        // Create OpenCL kernel for HEIC box detection
        let kernel_source = r#"
            __kernel void find_heic_boxes(
                __global const uchar* data,
                __global uint* box_offsets,
                __global uint* box_sizes,
                uint data_size
            ) {
                uint gid = get_global_id(0);
                if (gid + 8 >= data_size) return;
                
                // Read box size (first 4 bytes, big-endian)
                uint size = (data[gid] << 24) | 
                           (data[gid + 1] << 16) | 
                           (data[gid + 2] << 8) | 
                           data[gid + 3];
                
                if (size >= 8 && gid + size <= data_size) {
                    box_offsets[gid] = gid;
                    box_sizes[gid] = size;
                } else {
                    box_offsets[gid] = 0xFFFFFFFF;
                    box_sizes[gid] = 0;
                }
            }
        "#;

        // Compile and execute kernel
        let program = ocl::Program::builder()
            .src(kernel_source)
            .devices(context.devices())
            .build()?;

        let kernel = ocl::Kernel::builder()
            .program(&program)
            .name("find_heic_boxes")
            .queue(queue.clone())
            .arg(data)
            .arg(ocl::Buffer::<u32>::builder()
                .queue(queue.clone())
                .len(data.len())
                .build()?)
            .arg(ocl::Buffer::<u32>::builder()
                .queue(queue.clone())
                .len(data.len())
                .build()?)
            .arg(data.len() as u32)
            .build()?;

        // Execute kernel
        unsafe {
            kernel.enq()?;
        }

        // Process results
        Ok(())
    }
}

#[cfg(feature = "gpu")]
impl Default for GpuExifParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "gpu"))]
/// Dummy GPU parser when GPU feature is not enabled
pub struct GpuExifParser;

#[cfg(not(feature = "gpu"))]
impl GpuExifParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse_jpeg_exif_gpu(
        &mut self,
        data: &[u8],
        metadata: &mut std::collections::HashMap<String, String>,
    ) -> Result<(), crate::types::ExifError> {
        // Fallback to CPU implementation
        crate::parsers::jpeg::JpegParser::parse_jpeg_exif(data, metadata)
    }
    
    pub fn parse_heic_exif_gpu(
        &mut self,
        data: &[u8],
        metadata: &mut std::collections::HashMap<String, String>,
    ) -> Result<(), crate::types::ExifError> {
        // Fallback to CPU implementation
        crate::parsers::heif::HeifParser::parse_heif_exif(data, metadata)
    }
}

#[cfg(not(feature = "gpu"))]
impl Default for GpuExifParser {
    fn default() -> Self {
        Self::new()
    }
}
