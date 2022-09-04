mod lcu;
// mod async_func;
// use anyhow::Result;
use lcu::{ApiClient, Summoner};
// use std::collections::HashMap;\

use iced::{
    button, executor, window, Application, Button, Column, Command, Container, Element, Settings,
    Subscription, Text,
};
use tokio::runtime::Handle;

#[derive(Default)]
pub struct MainUI {
    refresh_button: button::State,
    account: Option<Summoner>,
    api: ApiClient,
}

fn fresh_summoner(api:ApiClient)->Option<Summoner>{
    let summ = tokio::task::block_in_place(move || {
        Handle::current().block_on(async move { api.summoner().await })
    });

    match summ {
        Ok(v) => Some(v),
        Err(e) => {
            println!("error:{}", e);
            None
        }
    }
}

impl MainUI {
    fn new() -> MainUI {
        let mut api = ApiClient::default();
        api.init_client();
        let ss = fresh_summoner(api);

        MainUI {
            refresh_button: button::State::new(),
            account: ss,
            api: ApiClient::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Send,
    Refresh,
}

impl Application for MainUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    fn new(_: ()) -> (Self, Command<Self::Message>) {
        (MainUI::new(), Command::none())
    }

    fn title(&self) -> String {
        String::from("一键喊话")
    }

    fn view(&mut self) -> Element<Message> {
        let cname_label = Text::new(self.account.clone().unwrap_or_default().displayName);

        let refresh_button =
            Button::new(&mut self.refresh_button, Text::new("refresh")).on_press(Message::Refresh);

        let content = Column::new().push(cname_label).push(refresh_button);
        Container::new(content).center_x().center_y().into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Send => {}
            Message::Refresh => {
                self.api.init_client();
                self.account = fresh_summoner(self.api.clone());
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

#[tokio::main]
async fn main() {
    let _ = MainUI::run(Settings {
        window: window::Settings {
            size: (440, 320),
            min_size: Some((200, 100)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: true,
            position: window::Position::Specific(5, 466),
            icon: None,
        },
        // flags: c,
        // default_font: Some(include_bytes!("C:/Windows/Fonts/SIMHEI.TTF")),
        ..Settings::default()
    });
}
