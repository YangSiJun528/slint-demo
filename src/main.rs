use std::rc::Rc;
use slint::{VecModel, ModelRc};

slint::include_modules!();

fn main() {
    let ui = Example::new().unwrap();
    ui.run().unwrap();
}
