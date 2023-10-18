use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Arc;

use ash::vk;
use gfx::Gfx;

use gfx::shader::{ShaderProgram};
use gfx::shader_instance::{ShaderInstance, ShaderInstanceCreateInfos};
use shader_base::pass_id::PassID;
use shader_base::{AlphaMode, BindPoint, CompilationError, Culling, FrontFace, PolygonMode, ShaderInterface, ShaderStage, Topology};
use shader_base::spirv_reflector::{DescriptorBinding, SpirvReflector};

//use crate::vk_types::VkPixelFormat;
use crate::{GfxVulkan, vk_check, VkShaderInstance};
use crate::vk_dst_set_layout::VkDescriptorSetLayout;
use crate::vk_types::VkPixelFormat;

//use crate::renderer::vk_render_pass::VkRenderPass;

pub struct VkTopology(vk::PrimitiveTopology);

pub struct VkPolygonMode(vk::PolygonMode);

pub struct VkCullMode(vk::CullModeFlags);

pub struct VkFrontFace(vk::FrontFace);

impl From<&Topology> for VkTopology {
    fn from(topology: &Topology) -> Self {
        VkTopology(match topology {
            Topology::Points => vk::PrimitiveTopology::POINT_LIST,
            Topology::Lines => vk::PrimitiveTopology::LINE_LIST,
            Topology::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST,
        })
    }
}

impl From<&PolygonMode> for VkPolygonMode {
    fn from(polygon_mode: &PolygonMode) -> Self {
        VkPolygonMode(match polygon_mode {
            PolygonMode::Point => vk::PolygonMode::POINT,
            PolygonMode::Line => vk::PolygonMode::LINE,
            PolygonMode::Fill => vk::PolygonMode::FILL,
        })
    }
}

impl From<&Culling> for VkCullMode {
    fn from(culling: &Culling) -> Self {
        VkCullMode(match culling {
            Culling::None => vk::CullModeFlags::NONE,
            Culling::Front => vk::CullModeFlags::FRONT,
            Culling::Back => vk::CullModeFlags::BACK,
            Culling::Both => vk::CullModeFlags::FRONT_AND_BACK,
        })
    }
}

impl From<&FrontFace> for VkFrontFace {
    fn from(culling: &FrontFace) -> Self {
        VkFrontFace(match culling {
            FrontFace::Clockwise => vk::FrontFace::CLOCKWISE,
            FrontFace::CounterClockwise => vk::FrontFace::COUNTER_CLOCKWISE,
        })
    }
}

pub struct VkShaderProgram {
    _vertex_module: Arc<VkShaderModule>,
    _fragment_module: Arc<VkShaderModule>,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: Arc<vk::PipelineLayout>,
    pub descriptor_set_layout: Arc<VkDescriptorSetLayout>,
    bindings: HashMap<BindPoint, DescriptorBinding>,
    name: String,
}

impl ShaderProgram for VkShaderProgram {
    fn get_bindings(&self) -> HashMap<BindPoint, DescriptorBinding> {
        self.bindings.clone()
    }

    fn instantiate(&self) -> Arc<dyn ShaderInstance> {
        VkShaderInstance::new(
            format!("{}_instance", self.name),
            ShaderInstanceCreateInfos {
                bindings: self.bindings.clone(),
            },
            self.pipeline_layout.clone(),
            self.descriptor_set_layout.clone(),
        )
    }
}

impl VkShaderProgram {
    pub fn new(
        name: String,
        pass_id: PassID,
        create_infos: &dyn ShaderInterface,
    ) -> Result<Arc<Self>, CompilationError> {
        let vertex_binary = create_infos.get_spirv_for(&pass_id, &ShaderStage::Vertex)?;
        let vertex_entry_point = create_infos.get_entry_point(&pass_id, &ShaderStage::Vertex).unwrap();
        let vertex_stage_input = match create_infos.get_stage_inputs(&pass_id, &ShaderStage::Vertex) {
            Ok(inputs) => { inputs }
            Err(err) => { return Err(CompilationError::throw(err, None)); }
        };
        let fragment_binary = create_infos.get_spirv_for(&pass_id, &ShaderStage::Fragment)?;
        let fragment_entry_point = create_infos.get_entry_point(&pass_id, &ShaderStage::Fragment).unwrap();
        let fragment_stage_outputs = match create_infos.get_stage_outputs(&pass_id, &ShaderStage::Fragment) {
            Ok(outputs) => { outputs }
            Err(err) => { return Err(CompilationError::throw(err, None)); }
        };
        let parameters = create_infos.get_parameters_for(&pass_id);

        let descriptor_set_layout = VkDescriptorSetLayout::new(
            name.clone(),
            &vertex_infos.bindings,
            &fragment_infos.bindings,
        );

        let mut bindings = vertex_infos.bindings.clone();
        for (key, value) in fragment_infos.bindings {
            if bindings.contains_key(&key) {
                logger::warning!("Binding duplication : {:?}", key);
            } else {
                bindings.insert(key.clone(), value.clone());
            }
        }

        let vertex_module = VkShaderModule::new(name.clone(), &vertex_binary);
        let fragment_module = VkShaderModule::new(name.clone(), &fragment_binary);

        let mut push_constants = Vec::<vk::PushConstantRange>::new();

        if let Some(vertex_pc_size) = vertex_infos.push_constant_size {
            push_constants.push(
                vk::PushConstantRange::builder()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .offset(0)
                    .size(vertex_pc_size)
                    .build(),
            );
        }
        if let Some(frag_pc_size) = fragment_infos.push_constant_size {
            push_constants.push(
                vk::PushConstantRange::builder()
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .offset(0)
                    .size(frag_pc_size)
                    .build(),
            );
        }

        let pipeline_layout_infos = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[descriptor_set_layout.descriptor_set_layout])
            .push_constant_ranges(push_constants.as_slice())
            .build();
        let pipeline_layout = Arc::new(GfxVulkan::get().set_vk_object_name(
            vk_check!(unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .create_pipeline_layout(&pipeline_layout_infos, None)
            }),
            format!("pipeline layout\t\t: {}", name).as_str(),
        ));

        let mut vertex_attribute_description = Vec::<vk::VertexInputAttributeDescription>::new();

        let mut vertex_input_size = 0;
        let mut property_location = 0;
        for input_property in &vertex_stage_input {
            vertex_attribute_description.push(
                vk::VertexInputAttributeDescription::builder()
                    .location(property_location)
                    .format(*VkPixelFormat::from(&input_property.format))
                    .offset(vertex_input_size)
                    .build(),
            );
            property_location += 1;
            vertex_input_size += input_property.format.type_size()
        }

        let mut binding_descriptions = Vec::new();
        if vertex_input_size > 0 {
            binding_descriptions.push(
                vk::VertexInputBindingDescription::builder()
                    .binding(0)
                    .stride(vertex_input_size)
                    .input_rate(vk::VertexInputRate::VERTEX)
                    .build(),
            );
        }

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions.as_slice())
            .vertex_attribute_descriptions(vertex_attribute_description.as_slice())
            .build();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(VkTopology::from(&parameters.topology).0)
            .primitive_restart_enable(false)
            .build();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .build();

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(VkPolygonMode::from(&parameters.polygon_mode).0)
            .cull_mode(VkCullMode::from(&parameters.culling).0)
            .front_face(VkFrontFace::from(&parameters.front_face).0)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .line_width(parameters.line_width)
            .build();

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(1.0)
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(parameters.depth_test)
            .depth_write_enable(parameters.depth_test)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .build();

        let mut color_blend_attachment = Vec::<vk::PipelineColorBlendAttachmentState>::new();

        let render_pass = Gfx::get().cast::<GfxVulkan>().render_pass_pool().find_by_id(&pass_id).unwrap();

        let mut writable_images = vec![];
        for image in render_pass.images.iter() {
            if !image.get_format().is_depth_format() { writable_images.push(image.clone()) }
        }

        if writable_images.len() != fragment_stage_outputs.len() {
            return Err(CompilationError::throw(format!("Fragment stage output ({}) does not match with render pass images ({})", fragment_stage_outputs.len(), writable_images.len()), None));
        }

        for (i, image) in writable_images.iter().enumerate() {
            if image.get_format().components() != fragment_stage_outputs[i].format.components() {
                return Err(CompilationError::throw(format!("Fragment stage output #{i} with format '{:?}' does not match color attachment #{i} of render pass {} : {:?}", fragment_stage_outputs[i].format, pass_id, image.get_format()), None));
            }
        }

        for image in &render_pass.images {
            if image.get_format().is_depth_format() { continue; }
            color_blend_attachment.push(
                vk::PipelineColorBlendAttachmentState::builder()
                    .blend_enable(parameters.alpha_mode != AlphaMode::Opaque)
                    .src_color_blend_factor(
                        if parameters.alpha_mode == AlphaMode::Opaque {
                            vk::BlendFactor::ZERO
                        } else {
                            vk::BlendFactor::SRC_ALPHA
                        },
                    )
                    .dst_color_blend_factor(
                        if parameters.alpha_mode == AlphaMode::Opaque {
                            vk::BlendFactor::ZERO
                        } else {
                            vk::BlendFactor::ONE_MINUS_SRC_ALPHA
                        },
                    )
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(
                        if parameters.alpha_mode == AlphaMode::Opaque {
                            vk::BlendFactor::ONE
                        } else {
                            vk::BlendFactor::ONE_MINUS_SRC_ALPHA
                        },
                    )
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(
                        vk::ColorComponentFlags::R
                            | vk::ColorComponentFlags::G
                            | vk::ColorComponentFlags::B
                            | vk::ColorComponentFlags::A,
                    )
                    .build(),
            );
        }

        let shader_stages = Vec::<vk::PipelineShaderStageCreateInfo>::from([
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module.get_module())
                .name(unsafe { CStr::from_ptr(vertex_entry_point.as_ptr() as *const c_char) })
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module.get_module())
                .name(unsafe { CStr::from_ptr(fragment_entry_point.as_ptr() as *const c_char) })
                .build(),
        ]);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(color_blend_attachment.as_slice())
            .build();

        let mut dynamic_states_array =
            Vec::from([vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT]);
        if parameters.line_width != 1.0 {
            dynamic_states_array.push(vk::DynamicState::LINE_WIDTH);
        }

        let dynamic_states = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic_states_array.as_slice())
            .build();

        let ci_pipeline = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages.as_slice())
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_states)
            .layout(*pipeline_layout)
            .render_pass(render_pass.render_pass)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::default())
            .base_pipeline_index(-1)
            .build();

        let pipeline = match unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .create_graphics_pipelines(vk::PipelineCache::default(), &[ci_pipeline], None)
        } {
            Ok(pipeline) => pipeline[0],
            Err(_) => {
                logger::fatal!("failed to create graphic pipelines")
            }
        };
        GfxVulkan::get()
            .set_vk_object_name(pipeline, format!("graphic pipeline\t\t: {}", name).as_str());

        Ok(Arc::new(Self {
            _vertex_module: vertex_module,
            _fragment_module: fragment_module,
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            bindings,
            name,
        }))
    }
}

pub struct VkShaderModule {
    shader_module: vk::ShaderModule,
}

impl VkShaderModule {
    pub fn new(name: String, spirv: &Vec<u32>) -> Arc<Self> {
        let ci_shader_module = vk::ShaderModuleCreateInfo::builder()
            .code(spirv.as_slice())
            .flags(vk::ShaderModuleCreateFlags::default())
            .build();

        let shader_module = vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .create_shader_module(&ci_shader_module, None)
        });
        GfxVulkan::get().set_vk_object_name(
            shader_module,
            format!("shader module\t\t: {}", name).as_str(),
        );

        Arc::new(Self { shader_module })
    }

    pub fn get_module(&self) -> vk::ShaderModule {
        self.shader_module
    }
}
