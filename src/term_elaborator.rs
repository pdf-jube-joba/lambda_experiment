use crate::kernel::Term;
use crate::surface::TermAST;
/*
// TermVar is defined as follows in crate::kernel
#[derive(Debug, Clone)]
pub struct TermVar(Rc<String>);

impl TermVar {
    pub fn new(name: &str) -> Self {
        TermVar(Rc::new(name.to_string()))
    }
    pub fn name(&self) -> &str {
        &self.0
    }
}
// variable is defined as `Term::Var(Term)`
*/

pub enum PendingEffect {
    ControlIdentifier { name: String },
    ControlAccess { module: String, name: String },
}

pub enum State {
    Done(Term),
    Target(TermAST),
    Pending(PendingEffect),
    Err(String),
}

pub struct TermScopeElaborator {
    // given
    reference_vars: Vec<crate::kernel::TermVar>,
    // state
    state: State,
    // extend in progress
    local_term_vars: Vec<crate::kernel::TermVar>,
    // stack frames of continuations
    frames: Vec<Box<dyn FnOnce(&mut TermScopeElaborator)>>,
}

impl TermScopeElaborator {
    pub fn elab_one_step(&mut self) {
        match &self.state {
            State::Done(_) => {
                if let Some(frame) = self.frames.pop() {
                    // Resume the next continuation
                    frame(self);
                } else {
                    // No more frames, elaboration is complete
                    return;
                }
            }
            State::Target(term_ast) => {
                match term_ast {
                    TermAST::Sort(sort) => todo!(),
                    TermAST::Identifier(v) => {
                        // find in local vars
                        if let Some(var) = self
                            .local_term_vars
                            .iter()
                            .rev()
                            .find(|var| var.name() == v)
                            .cloned()
                        {
                            self.state = State::Done(Term::Var(var));
                            return;
                        }

                        // find in reference vars
                        if let Some(var) = self
                            .reference_vars
                            .iter()
                            .rev()
                            .find(|var| var.name() == v)
                            .cloned()
                        {
                            self.state = State::Done(Term::Var(var));
                            return;
                        }

                        // not found => pending effect
                        self.state =
                            State::Pending(PendingEffect::ControlIdentifier { name: v.clone() });
                    }
                    TermAST::Access { module, name } => todo!(),
                    TermAST::Prod {
                        param,
                        param_type,
                        body,
                    } => {
                        let param_var = crate::kernel::TermVar::new(&param);
                        let param_type = *param_type.clone();
                        let body = *body.clone();

                        // Push a continuation to handle the body after elaborating the param_type
                        self.frames.push(Box::new(move |elab| {
                            // Add the parameter to the local scope
                            elab.local_term_vars.push(param_var.clone());

                            // Push another continuation to finalize the Prod term
                            elab.frames.push(Box::new(move |elab| {
                                // Pop the parameter from the local scope
                                elab.local_term_vars.pop();

                                // Combine the elaborated parameter type and body into a Prod term
                                if let State::Done(body) = &elab.state {
                                    elab.state = State::Done(Term::Prod {
                                        param: param_var,
                                        param_type: Box::new(param_type),
                                        body: Box::new(body.clone()),
                                    });
                                } else {
                                    elab.state =
                                        State::Err("Unexpected state in Prod finalization".to_string());
                                }
                            }));

                            // Set the state to elaborate the body
                            elab.state = State::Target(body);
                        }));

                        // Set the state to elaborate the param_type
                        self.state = State::Target(param_type);
                    }
                    TermAST::Abs {
                        param,
                        param_type,
                        body,
                    } => todo!(),
                    TermAST::App { func, arg } => todo!(),
                    TermAST::Nat => todo!(),
                    TermAST::Zero => todo!(),
                    TermAST::Succ(term_ast) => todo!(),
                    TermAST::PrimitiveRecursion {
                        motive,
                        zero_case,
                        succ_case,
                        n,
                    } => todo!(),
                }
            }
            State::Pending(pending_effect) => todo!(),
            State::Err(_) => todo!(),
        }
    }
}
