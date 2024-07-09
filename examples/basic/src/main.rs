use my_gui::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::default();
    app.set_bg_color(&[0.824, 0.902, 0.698, 1.0]);
    app.set_title("My Test GUI Kit");

    app.run()
}
