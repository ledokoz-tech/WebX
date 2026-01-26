use iced::{widget::container, Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    // WebView messages will go here
}

pub struct BrowserView {
    // WebView integration will go here
}

impl BrowserView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self) -> Element<Message> {
        container(
            iced::widget::text("WebView integration coming soon...")
                .size(16)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
