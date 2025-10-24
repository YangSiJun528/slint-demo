slint::include_modules!();

pub fn main() {
    let main_window = MainWindow::new().unwrap();
    main_window.run().unwrap();
}