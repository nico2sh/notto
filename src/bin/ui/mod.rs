mod note_browser;
pub mod error;

use crossbeam_channel::{Receiver, Sender};
use cursive::{Cursive, CursiveExt, View, event::{Event, EventResult}, views::{Dialog, LinearLayout}};

use self::error::NottoViewError;

pub enum UIMessage {

}

pub trait NottoScreen {
    fn load_view(&self) -> Result<Box<dyn View>, NottoViewError>;
}

pub struct NottoUI {
    siv: Cursive,
    rx: Receiver<UIMessage>,
    tx: Sender<UIMessage>
}

impl NottoUI {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let cursive = Cursive::default();
        Self { siv: cursive, rx, tx }
    }

    fn run(&mut self) {
        self.build_view();
        self.siv.run();
    }

    fn build_view(&mut self) {
        self.siv.add_layer(
            Dialog::around(
                LinearLayout::horizontal()
            )
        )
    }
}

impl cursive::view::View for NottoUI {
    fn draw(&self, _printer: &cursive::Printer) {
        // do nothing
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        EventResult::Ignored
    }
}