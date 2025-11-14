use std::rc::Rc;

use crate::{
    kernel::{Context, DefinedConstant, Term, TermVar},
    surface::{self, AccessPath},
};

pub enum ModuleItem {
    Definition {
        name: String,
        rc: Rc<crate::kernel::DefinedConstant>,
    },
}

// this is after elaboration but before instantiation
// i.e., template module with parameters
// user cannot see this
pub struct ModuleElaborated {
    pub name: String,
    pub parameters: Vec<(String, Term)>,
    pub parent: Option<usize>,
    pub child_modules: Vec<usize>,
    pub body: Vec<ModuleItem>,
}

// this is after instantiation
// i.e., concrete module with parameters instantiated
// user accesses this
pub struct ModuleInstantiated {
    pub name: String,
    pub body: Vec<ModuleItem>,
}

fn instantiate_module(
    elaborated: &ModuleElaborated,
    args: &[(String, Term)],
) -> Result<ModuleInstantiated, String> {
    // build substitution map
    let mut subst_map = Vec::new();
    for ((param_name, _), (_, arg_term)) in elaborated.parameters.iter().zip(args.iter()) {
        subst_map.push((TermVar::new(param_name), arg_term.clone()));
    }

    // instantiate body
    let mut instantiated_body = Vec::new();
    for item in &elaborated.body {
        match item {
            ModuleItem::Definition { name, rc } => {
                let DefinedConstant { name: _, ty, term } = rc.as_ref();
                let ty_subst = crate::kernel::substitute_map(ty, &subst_map);
                let term_subst = crate::kernel::substitute_map(term, &subst_map);
                let instantiated_rc = Rc::new(DefinedConstant {
                    name: name.clone(),
                    ty: ty_subst,
                    term: term_subst,
                });
                instantiated_body.push(ModuleItem::Definition {
                    name: name.clone(),
                    rc: instantiated_rc,
                });
            }
        }
    }

    Ok(ModuleInstantiated {
        name: elaborated.name.clone(),
        body: instantiated_body,
    })
}

// tree of elaborated modules with current module pointer
pub struct ModuleSystem {
    // root is 0th module
    pub modules: Vec<ModuleElaborated>,
    pub current: usize,
}

pub enum AccessPathElab {
    Parent(usize, Vec<ModulePathFrameElab>),
    Root(Vec<ModulePathFrameElab>),
}

pub struct ModulePathFrameElab {
    pub name: String,
    pub arguments: Vec<(String, Term)>,
}

impl ModuleSystem {
    pub fn current_ctx(&self) -> Context {
        let mut ctx = Vec::new();
        let mut module_idx = self.current;
        loop {
            let module = &self.modules[module_idx];
            for (param_name, param_ty) in &module.parameters {
                ctx.push((TermVar::new(param_name), param_ty.clone()));
            }
            match module.parent {
                Some(parent_idx) => {
                    module_idx = parent_idx;
                }
                None => break,
            }
        }
        ctx.reverse();
        ctx
    }

    // returns the list of instantiated modules along the access path
    pub fn access_path_current(
        &self,
        path: &AccessPathElab,
    ) -> Result<(ModuleInstantiated, Vec<surface::PendingEffect>), String> {
        // use PendingEffect::TypeCheck to represent type checking tasks
        //   when instantiating modules along the access path (v[i] := term[i]) of type ty[i]

        // 1. first, resolve where to start
        let (mut module_idx, frames) = match path {
            AccessPathElab::Parent(up, frames) => {
                let mut idx = self.current;
                for _ in 0..*up {
                    idx = self.modules[idx]
                        .parent
                        .ok_or("Cannot go up from root module")?;
                }
                (idx, frames)
            }
            AccessPathElab::Root(frames) => (0, frames),
        };

        // 2. then, get a corresponding elaborated module (templates) along the path
        //   and prepare type checking tasks for instantiation
        let mut subst_frames: Vec<Vec<(String, Term)>> = Vec::new();
        let mut pending_effects_frames: Vec<Vec<surface::PendingEffect>> = Vec::new();

        for frame in frames {
            let module_elab = &self.modules[module_idx];
            // find child module with the given name
            let child_module_idx = module_elab
                .child_modules
                .iter()
                .find(|&&child_idx| self.modules[child_idx].name == frame.name)
                .ok_or(format!(
                    "Module '{}' not found in module '{}'",
                    frame.name, module_elab.name
                ))?;
            let child_module_elab = &self.modules[*child_module_idx];

            // check that the number of arguments matches
            if frame.arguments.len() != child_module_elab.parameters.len() {
                return Err(format!(
                    "Module '{}' expects {} arguments, but {} were provided",
                    child_module_elab.name,
                    child_module_elab.parameters.len(),
                    frame.arguments.len()
                ));
            }

            // prepare substitution map for this frame
            let mut subst_args = Vec::new();
            let mut pending_effects = Vec::new();
            for ((param_name, param_ty), (arg_name, arg_term)) in child_module_elab
                .parameters
                .iter()
                .zip(frame.arguments.iter())
            {
                if param_name != arg_name {
                    return Err(format!(
                        "Parameter name mismatch: expected '{}', found '{}'",
                        param_name, arg_name
                    ));
                }
                // add to substitution args
                subst_args.push((param_name.clone(), arg_term.clone()));
                // add type checking task
                pending_effects.push(surface::PendingEffect::TypeCheck(
                    Vec::new(),
                    vec![(TermVar::new(arg_name), arg_term.clone(), param_ty.clone())],
                ));
            }

            // store for later instantiation
            subst_frames.push(subst_args);
            pending_effects_frames.push(pending_effects);

            // move to child module
            module_idx = *child_module_idx;
        }

        // 3. from the elaborated module, instantiate a concrete module using the prepared arguments
        let subst_map = subst_frames
            .iter()
            .flat_map(|args| args.clone())
            .collect::<Vec<_>>();
        let instantiated_module = instantiate_module(&self.modules[module_idx], &subst_map)?;
        let pending_effects = pending_effects_frames
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok((instantiated_module, pending_effects))
    }
}
