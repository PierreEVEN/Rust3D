use std::path::PathBuf;
use shader_base::{CompilationError, ShaderStage};

#[derive(Default)]
pub struct HlslToSpirv {}

impl HlslToSpirv {
    pub fn transpile(
        &self,
        hlsl: &String,
        entry_point: &String,
        virtual_path: &PathBuf,
        shader_stage: &ShaderStage,
    ) -> Result<Vec<u32>, CompilationError> {
        logger::info!("shader :\n{hlsl}");
        let compiler = match shaderc::Compiler::new() {
            None => { return Err(CompilationError::throw("failed to create shaderc compiler".to_string(), None)); }
            Some(compiler) => compiler,
        };

        let mut compile_options = match shaderc::CompileOptions::new() {
            None => { return Err(CompilationError::throw("failed to create shaderc compile option".to_string(), None)); }
            Some(compiler) => compiler,
        };
        compile_options.set_hlsl_io_mapping(true);
        compile_options.set_auto_map_locations(true);
        compile_options.set_target_env(shaderc::TargetEnv::Vulkan, shaderc::EnvVersion::Vulkan1_2 as u32);
        compile_options.set_target_spirv(shaderc::SpirvVersion::V1_3);
        compile_options.set_source_language(shaderc::SourceLanguage::HLSL);

        let binary_result = match compiler.compile_into_spirv(
            &hlsl,
            match shader_stage {
                ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
                ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
                ShaderStage::TesselationControl => shaderc::ShaderKind::TessControl,
                ShaderStage::TesselationEvaluate => shaderc::ShaderKind::TessEvaluation,
                ShaderStage::Geometry => { shaderc::ShaderKind::Geometry }
                ShaderStage::Compute => { shaderc::ShaderKind::Compute }
            },
            virtual_path.to_str().unwrap(),
            entry_point.as_str(),
            Some(&compile_options),
        ) {
            Ok(binary) => binary,
            Err(compile_error) => {
                return Err(CompilationError::throw(format!("{:?}", compile_error), None))
            }
        };
        Ok(Vec::from(binary_result.as_binary()))
    }
}
