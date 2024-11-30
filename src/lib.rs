pub mod core;
pub mod event_system;

use core::{logger::init_logger, runner::applications::Application};

#[no_mangle]
pub extern "C" fn run() {
    init_logger();
    let mut app = Application::default();
    app.run();
}
