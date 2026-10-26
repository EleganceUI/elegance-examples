// A prop can be any `Clone + 'static` type. A setter takes `impl Into<T>`, so a `From` impl
// decides what callers may pass: here, any closure becomes an `Action`.

use std::rc::Rc;

#[derive(Clone)]
pub struct Action(Rc<dyn Fn()>);

impl Action {
    pub fn run(&self) {
        (self.0)()
    }
}

impl<F: Fn() + 'static> From<F> for Action {
    fn from(action: F) -> Self {
        Action(Rc::new(action))
    }
}
