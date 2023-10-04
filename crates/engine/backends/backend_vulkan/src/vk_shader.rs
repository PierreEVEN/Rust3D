use std::ffi::CStr;
use std::os::raw::c_char;
//use std::ffi::CStr;
//use std::os::raw::c_char;
use std::sync::Arc;

use ash::vk;
use gfx::Gfx;

use gfx::shader::{AlphaMode, Culling, DescriptorBinding, FrontFace, PassID, PolygonMode, ShaderProgram, ShaderProgramInfos, Topology};
use gfx::shader_instance::{ShaderInstance, ShaderInstanceCreateInfos};

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
    bindings: Vec<DescriptorBinding>,
    name: String,
}

impl ShaderProgram for VkShaderProgram {
    fn get_bindings(&self) -> Vec<DescriptorBinding> {
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
        create_infos: &ShaderProgramInfos,
    ) -> Arc<Self> {
        let descriptor_set_layout = VkDescriptorSetLayout::new(
            name.clone(),
            &create_infos.vertex_stage.descriptor_bindings,
            &create_infos.fragment_stage.descriptor_bindings,
        );

        let mut bindings = create_infos.vertex_stage.descriptor_bindings.clone();
        bindings.append(&mut create_infos.fragment_stage.descriptor_bindings.clone());

        let vertex_module = VkShaderModule::new(name.clone(), &create_infos.vertex_stage.spirv);
        let fragment_module = VkShaderModule::new(name.clone(), &create_infos.fragment_stage.spirv);

        let mut push_constants = Vec::<vk::PushConstantRange>::new();

        if create_infos.vertex_stage.push_constant_size > 0 {
            push_constants.push(
                vk::PushConstantRange::builder()
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .offset(0)
                    .size(create_infos.vertex_stage.push_constant_size)
                    .build(),
            );
        }
        if create_infos.fragment_stage.push_constant_size > 0 {
            push_constants.push(
                vk::PushConstantRange::builder()
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .offset(0)
                    .size(create_infos.fragment_stage.push_constant_size)
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

        for input_property in &create_infos.vertex_stage.stage_input {
            if input_property.location < 0 {
                continue;
            }

            vertex_attribute_description.push(
                vk::VertexInputAttributeDescription::builder()
                    .location(input_property.location as _)
                    .format(*VkPixelFormat::from(&input_property.property_type.format))
                    .offset(input_property.offset)
                    .build(),
            );

            vertex_input_size += input_property.property_type.format.type_size()
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
            .topology(VkTopology::from(&create_infos.shader_properties.topology).0)
            .primitive_restart_enable(false)
            .build();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .build();

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(VkPolygonMode::from(&create_infos.shader_properties.polygon_mode).0)
            .cull_mode(VkCullMode::from(&create_infos.shader_properties.culling).0)
            .front_face(VkFrontFace::from(&create_infos.shader_properties.front_face).0)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .line_width(create_infos.shader_properties.line_width)
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
            .depth_test_enable(create_infos.shader_properties.depth_test)
            .depth_write_enable(create_infos.shader_properties.depth_test)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .build();

        let mut color_blend_attachment = Vec::<vk::PipelineColorBlendAttachmentState>::new();
        
        let render_pass = Gfx::get().cast::<GfxVulkan>().render_pass_pool().find_by_id(&pass_id).unwrap();
        for _ in &render_pass.images {
            color_blend_attachment.push(
                vk::PipelineColorBlendAttachmentState::builder()
                    .blend_enable(create_infos.shader_properties.alpha_mode != AlphaMode::Opaque)
                    .src_color_blend_factor(
                        if create_infos.shader_properties.alpha_mode == AlphaMode::Opaque {
                            vk::BlendFactor::ZERO
                        } else {
                            vk::BlendFactor::SRC_ALPHA
                        },
                    )
                    .dst_color_blend_factor(
                        if create_infos.shader_properties.alpha_mode == AlphaMode::Opaque {
                            vk::BlendFactor::ZERO
                        } else {
                            vk::BlendFactor::ONE_MINUS_SRC_ALPHA
                        },
                    )
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(
                        if create_infos.shader_properties.alpha_mode == AlphaMode::Opaque {
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
                .name(unsafe { CStr::from_ptr("main\0".as_ptr() as *const c_char) })
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module.get_module())
                .name(unsafe { CStr::from_ptr("main\0".as_ptr() as *const c_char) })
                .build(),
        ]);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(color_blend_attachment.as_slice())
            .build();

        let mut dynamic_states_array =
            Vec::from([vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT]);
        if create_infos.shader_properties.line_width != 1.0 {
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

        Arc::new(Self {
            _vertex_module: vertex_module,
            _fragment_module: fragment_module,
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            bindings,
            name,
        })
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
