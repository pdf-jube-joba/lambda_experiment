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

pub enum InnerCall {
    // continuation of elaborating Prod param type
    ProdLeft { param: String, body: TermAST },
    // continuation of elaborating Prod body
    ProdRight { param: String, param_type: Term },
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
    frames: Vec<InnerCall>,
}

impl TermScopeElaborator {
    pub fn elab_one_step(&mut self) {
        match &self.state {
            State::Done(_) => {
                let Some(frame) = self.frames.pop() else {
                    // finished
                    return;
                };
                todo!()
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
                        // push a frame to continue after elaborating param_type
                        self.frames.push(InnerCall::ProdLeft {
                            param: param.clone(),
                            body: *body.clone(),
                        });
                        // set state to elaborate param_type
                        self.state = State::Target(*param_type.clone());
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
