use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use shader_base::pass_id::PassID;
use shader_base::{ShaderInterface};
use crate::Gfx;
use crate::material::MaterialResourcePool;
use crate::shader::ShaderProgram;

#[derive(Hash, Eq, PartialEq)]
struct ProgramUID {
    pass_id: PassID,
    path: PathBuf,
}

impl ProgramUID {
    pub fn new(pass_id: PassID, path: PathBuf) -> Self { Self { pass_id, path } }
}

#[derive(Default)]
pub struct ProgramPool {
    existing_programs: HashMap<ProgramUID, Arc<dyn ShaderProgram>>,
}

impl ProgramPool {
    pub fn find_or_create_program(&self, pass_id: &PassID, shi: &Arc<dyn ShaderInterface>, resources: &Arc<MaterialResourcePool>) -> Option<Arc<dyn ShaderProgram>> {
        let program_uid = ProgramUID::new(pass_id.clone(), shi.get_path());
        match self.existing_programs.get(&program_uid) {
            None => {
                match Gfx::get().create_shader_program(shi.get_path().to_str().unwrap().to_string(), pass_id.clone(), shi.as_ref(), resources.clone()) {
                    Ok(program) => { Some(program) }
                    Err(compilation_error) => {
                        logger::warning!("Failed to compile shader {:?} :\n{:?}", shi.get_path(), compilation_error);
                        None
                    }
                }
            }
            Some(existing) => { Some(existing.clone()) }
        }
    }
}