﻿use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use logger::{fatal};
use maths::vec2::Vec2u32;
use crate::renderer::renderer_resource::PassResource;

/// This is a single node of a render graph. 
/// It contains an array of nodes that will require the resources generated by this pass
/// In other words :
/// ```
/// use gfx::renderer::render_node::RenderNode;
/// 
/// let mut parent = RenderNode::default();
/// let child = RenderNode::default();
/// parent.attach(std::sync::Arc::new(child))
/// ```
/// will make child.resources available to parent
pub struct RenderNode {
    name: String,
    inputs: Vec<Arc<RenderNode>>,
    resources: Vec<PassResource>,
    compute_res: Arc<RwLock<dyn FnMut(Vec2u32) -> Vec2u32>>,
    present_pass: bool,
}

impl Default for RenderNode {
    fn default() -> Self {
        Self {
            name: "unknown_pass".to_string(),
            inputs: vec![],
            resources: vec![],
            compute_res: Arc::new(RwLock::new(|res| { res })),
            present_pass: false,
        }
    }
}

impl RenderNode {
    /// Create a default Render Node for present usage
    pub fn present() -> Self {
        Self {
            name: "PresentPass".to_string(),
            inputs: vec![],
            resources: vec![],
            compute_res: Arc::new(RwLock::new(|res| { res })),
            present_pass: true,
        }
    }
    
    /// Retrieve resources of this pass
    pub fn resources(&self) -> &Vec<PassResource> {
        &self.resources
    }
    
    pub fn is_present_pass(&self) -> bool { self.present_pass }

    pub fn attach(&mut self, previous: Arc<RenderNode>) {
        if previous.present_pass {
            fatal!("Present pass cannot be attached to parent pass");
        }
        self.inputs.push(previous);
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
    
    pub fn res_override<H: FnMut(Vec2u32) -> Vec2u32 + 'static>(mut self, func: H) {
        self.compute_res = Arc::new(RwLock::new(func));
    }

    pub fn add_resource(mut self, resource: PassResource) -> Self {
        self.resources.push(resource);
        self
    }
    
    pub fn compute_res(&self) -> &Arc<RwLock<dyn FnMut(Vec2u32) -> Vec2u32>> {
        &self.compute_res        
    }

    pub fn inputs(&self) -> &Vec<Arc<RenderNode>> {
        &self.inputs
    }
}

impl Hash for RenderNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.resources.hash(state);
    }
}

impl PartialEq for RenderNode {
    fn eq(&self, other: &Self) -> bool {
        self.resources == other.resources
    }
}

impl Eq for RenderNode {}