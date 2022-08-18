use std::mem;
use std::os::raw::c_char;
use std::ptr::null;
use std::sync::Arc;

use ash::vk::{Bool32, CompareOp, CullModeFlags, DynamicState, GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PrimitiveTopology, PushConstantRange, RenderPass, SampleCountFlags, ShaderModule, ShaderModuleCreateFlags, ShaderModuleCreateInfo, ShaderStageFlags, StructureType, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate};
use gfx::GfxRef;

use gfx::shader::{Culling, FrontFace, PolygonMode, ShaderProgramInfos, Topology};

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};
use crate::vk_descriptor_set::VkDescriptorSetLayout;
use crate::vk_types::VkPixelFormat;

pub struct VkTopology(PrimitiveTopology);

pub struct VkPolygonMode(ash::vk::PolygonMode);

pub struct VkCullMode(CullModeFlags);

pub struct VkFrontFace(ash::vk::FrontFace);

impl From<&Topology> for VkTopology {
    fn from(topology: &Topology) -> Self {
        VkTopology(match topology {
            Topology::Points => { PrimitiveTopology::POINT_LIST }
            Topology::Lines => { PrimitiveTopology::LINE_LIST }
            Topology::Triangles => { PrimitiveTopology::TRIANGLE_LIST }
        })
    }
}

impl From<&PolygonMode> for VkPolygonMode {
    fn from(polygon_mode: &PolygonMode) -> Self {
        VkPolygonMode(match polygon_mode {
            PolygonMode::Point => { ash::vk::PolygonMode::POINT }
            PolygonMode::Line => { ash::vk::PolygonMode::LINE }
            PolygonMode::Fill => { ash::vk::PolygonMode::FILL }
        })
    }
}

impl From<&Culling> for VkCullMode {
    fn from(culling: &Culling) -> Self {
        VkCullMode(match culling {
            Culling::None => { CullModeFlags::NONE }
            Culling::Front => { CullModeFlags::FRONT }
            Culling::Back => { CullModeFlags::BACK }
            Culling::Both => { CullModeFlags::FRONT_AND_BACK }
        })
    }
}

impl From<&FrontFace> for VkFrontFace {
    fn from(culling: &FrontFace) -> Self {
        VkFrontFace(match culling {
            FrontFace::Clockwise => { ash::vk::FrontFace::CLOCKWISE }
            FrontFace::CounterClockwise => { ash::vk::FrontFace::COUNTER_CLOCKWISE }
        })
    }
}

pub struct VkShaderProgram {
    _gfx: GfxRef,
    _vertex_module: Arc<VkShaderModule>,
    _fragment_module: Arc<VkShaderModule>,
    _pipeline: Pipeline,
    _pipeline_layout: PipelineLayout,
    _descriptor_set_layout: Arc<VkDescriptorSetLayout>,
}

impl VkShaderProgram {
    pub fn new(gfx: &GfxRef, create_infos: &ShaderProgramInfos, descriptor_set_layout: &Arc<VkDescriptorSetLayout>) -> Arc<Self> {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        
        let vertex_module = VkShaderModule::new(gfx, &create_infos.vertex_stage.spirv);
        let fragment_module = VkShaderModule::new(gfx, &create_infos.fragment_stage.spirv);

        let mut push_constants = Vec::<PushConstantRange>::new();

        if create_infos.vertex_stage.push_constant_size > 0
        {
            push_constants.push(PushConstantRange {
                stage_flags: ShaderStageFlags::VERTEX,
                offset: 0,
                size: create_infos.vertex_stage.push_constant_size,
                ..PushConstantRange::default()
            });
        }
        if create_infos.fragment_stage.push_constant_size > 0
        {
            push_constants.push(PushConstantRange {
                stage_flags: ShaderStageFlags::FRAGMENT,
                offset: 0,
                size: create_infos.fragment_stage.push_constant_size,
            });
        }

        let pipeline_layout_infos = PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &descriptor_set_layout.descriptor_set_layout,
            push_constant_range_count: push_constants.len() as u32,
            p_push_constant_ranges: push_constants.as_ptr(),
            ..PipelineLayoutCreateInfo::default()
        };
        let pipeline_layout = vk_check!(unsafe { gfx_object!(*device).device.create_pipeline_layout(&pipeline_layout_infos, None) });

        let mut vertex_attribute_description = Vec::<VertexInputAttributeDescription>::new();

        let mut vertex_input_size = 0;

        for input_property in &create_infos.vertex_stage.stage_input
        {
            if input_property.location < 0 {
                continue;
            }

            vertex_attribute_description.push(VertexInputAttributeDescription {
                location: input_property.location as u32,
                format: *VkPixelFormat::from(&input_property.property_type.format),
                offset: input_property.offset,
                ..VertexInputAttributeDescription::default()
            });

            vertex_input_size += input_property.property_type.format.type_size()
        }

        let binding_descriptions = VertexInputBindingDescription {
            binding: 0,
            stride: vertex_input_size,
            input_rate: VertexInputRate::VERTEX,
        };

        let vertex_input_state = PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: if binding_descriptions.stride > 0 { 1 } else { 0 },
            p_vertex_binding_descriptions: if binding_descriptions.stride > 0 { &binding_descriptions } else { null() },
            vertex_attribute_description_count: vertex_attribute_description.len() as u32,
            p_vertex_attribute_descriptions: vertex_attribute_description.as_ptr(),
            ..PipelineVertexInputStateCreateInfo::default()
        };


        let input_assembly = PipelineInputAssemblyStateCreateInfo {
            topology: VkTopology::from(&create_infos.topology).0,
            primitive_restart_enable: false as Bool32,
            ..PipelineInputAssemblyStateCreateInfo::default()
        };

        let viewport_state = PipelineViewportStateCreateInfo {
            viewport_count: 1,
            scissor_count: 1,
            ..PipelineViewportStateCreateInfo::default()
        };

        let rasterizer = PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: false as Bool32,
            rasterizer_discard_enable: false as Bool32,
            polygon_mode: VkPolygonMode::from(&create_infos.polygon_mode).0,
            cull_mode: VkCullMode::from(&create_infos.culling).0,
            front_face: VkFrontFace::from(&create_infos.front_face).0,
            depth_bias_enable: false as Bool32,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: create_infos.line_width,
            ..PipelineRasterizationStateCreateInfo::default()
        };


        let multisampling = PipelineMultisampleStateCreateInfo {
            rasterization_samples: SampleCountFlags::TYPE_1,
            sample_shading_enable: false as Bool32,
            min_sample_shading: 1.0,
            p_sample_mask: null(),
            alpha_to_coverage_enable: false as Bool32,
            alpha_to_one_enable: false as Bool32,
            ..PipelineMultisampleStateCreateInfo::default()
        };


        let depth_stencil = PipelineDepthStencilStateCreateInfo {
            depth_test_enable: create_infos.depth_test as Bool32,
            depth_write_enable: create_infos.depth_test as Bool32,
            depth_compare_op: CompareOp::LESS,
            depth_bounds_test_enable: false as Bool32,
            stencil_test_enable: false as Bool32,
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
            ..PipelineDepthStencilStateCreateInfo::default()
        };

        let color_blend_attachment = Vec::<PipelineColorBlendAttachmentState>::new();

        /*
        for i in 0..render_pass.color_attachment_count
        {
            color_blend_attachment.push(PipelineColorBlendAttachmentState {
                blend_enable: if create_infos.alpha_mode == AlphaMode::Opaque { false } else { true } as Bool32,
                src_color_blend_factor: if create_infos.alpha_mode == AlphaMode::Opaque { BlendFactor::ZERO } else { BlendFactor::SRC_ALPHA },
                dst_color_blend_factor: if create_infos.alpha_mode == AlphaMode::Opaque { BlendFactor::ZERO } else { BlendFactor::ONE_MINUS_SRC_ALPHA },
                color_blend_op: BlendOp::ADD,
                src_alpha_blend_factor: if create_infos.alpha_mode == AlphaMode::Opaque { BlendFactor::ONE } else { BlendFactor::ONE_MINUS_SRC_ALPHA },
                dst_alpha_blend_factor: BlendFactor::ZERO,
                alpha_blend_op: BlendOp::ADD,
                color_write_mask: ColorComponentFlags::R | ColorComponentFlags::G | ColorComponentFlags::B | ColorComponentFlags::A,
                ..PipelineColorBlendAttachmentState::default()
            });
        }
        */

        let shader_stages = Vec::<PipelineShaderStageCreateInfo>::from([
            PipelineShaderStageCreateInfo {
                stage: ShaderStageFlags::VERTEX,
                module: vertex_module.get_module(),
                p_name: "main".as_ptr() as *const c_char,
                ..PipelineShaderStageCreateInfo::default()
            },
            PipelineShaderStageCreateInfo {
                stage: ShaderStageFlags::FRAGMENT,
                module: fragment_module.get_module(),
                p_name: "main".as_ptr() as *const c_char,
                ..PipelineShaderStageCreateInfo::default()
            }
        ]);


        let color_blending = PipelineColorBlendStateCreateInfo {
            attachment_count: color_blend_attachment.len() as u32,
            p_attachments: color_blend_attachment.as_ptr(),
            ..PipelineColorBlendStateCreateInfo::default()
        };


        let mut dynamic_states_array = Vec::from([DynamicState::SCISSOR, DynamicState::VIEWPORT]);
        if create_infos.line_width != 1.0 {
            dynamic_states_array.push(DynamicState::LINE_WIDTH);
        }


        let dynamic_states = PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states_array.len() as u32,
            p_dynamic_states: dynamic_states_array.as_ptr(),
            ..PipelineDynamicStateCreateInfo::default()
        };


        let ci_pipeline = GraphicsPipelineCreateInfo {
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state,
            p_input_assembly_state: &input_assembly,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisampling,
            p_depth_stencil_state: &depth_stencil,
            p_color_blend_state: &color_blending,
            p_dynamic_state: &dynamic_states,
            layout: pipeline_layout,
            render_pass: RenderPass::default(),//render_pass.get_render_pass(),
            subpass: 0,
            base_pipeline_handle: Pipeline::default(),
            base_pipeline_index: -1,
            ..GraphicsPipelineCreateInfo::default()
        };

        let pipeline = match unsafe { gfx_object!(*device).device.create_graphics_pipelines(PipelineCache::default(), &[ci_pipeline], None) } {
            Ok(pipeline) => { pipeline[0] }
            Err(_) => { panic!("failed to create graphic pipelines") }
        };


        Arc::new(Self {
            _gfx: gfx.clone(),
            _vertex_module: vertex_module,
            _fragment_module: fragment_module,
            _pipeline: pipeline,
            _pipeline_layout: pipeline_layout,
            _descriptor_set_layout: descriptor_set_layout.clone()
        })
    }
}

pub struct VkShaderModule {
    _gfx: GfxRef,
    shader_module: ShaderModule,
}

impl VkShaderModule {
    pub fn new(gfx: &GfxRef, spirv: &Vec<u32>) -> Arc<Self> {

        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        
        let ci_shader_module = ShaderModuleCreateInfo {
            s_type: StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: spirv.len() * mem::size_of::<u32>(),
            p_code: spirv.as_ptr(),
            flags: ShaderModuleCreateFlags::default(),
            p_next: null(),
        };

        let shader_module = vk_check!(unsafe { gfx_object!(*device).device.create_shader_module(&ci_shader_module, None) });

        Arc::new(Self {
            _gfx: gfx.clone(),
            shader_module,
        })
    }

    pub fn get_module(&self) -> ShaderModule {
        self.shader_module
    }
}