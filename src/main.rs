mod handler;
mod window;
mod vfs;

use window::polina_vfs::MainWindow;

pub fn main() -> iced::Result {
    iced::application("Polina VFS", MainWindow::update, MainWindow::view)
        .window_size(iced::Size::new(600.0, 800.0))
        .run()
}
