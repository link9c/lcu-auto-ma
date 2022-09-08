// #![windows_subsystem = "windows"]
mod lcu;
mod worker;
use iced::Alignment;
use iced::Font;
// use anyhow::Result;
use lcu::entity::LCUpackage;
use lcu::entity::Summoner;
use worker::{Message, WorkEvent, WorkInput, WorkMap, WorkerSender};
// use std::collections::HashMap;\

use iced::{
    button, executor, window, Application, Button, Column, Command, Container, Element, Length,
    Row, Settings, Subscription, Text,
};

#[derive(Default)]
pub struct MainUI {
    refresh_button: button::State,
    account: Option<Summoner>,
    // api: ApiClient,
    worker_list: Vec<WorkMap>,
    work_sender: WorkerSender,
}

impl MainUI {
    fn new() -> MainUI {
        MainUI {
            refresh_button: button::State::new(),
            account: None,
            // api: ApiClient::default(),
            worker_list: vec![WorkMap::new(0)],
            work_sender: WorkerSender::default(),
        }
    }
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

        // const XQFONT: Font = Font::External {
        //     name: "方正字体",
        //     bytes: include_bytes!("C:/Windows/Fonts/SIMYOU.TTF"), // 用 include_bytes 如果路径错误，会提示的
        // };
        let refresh_button =
        Button::new(&mut self.refresh_button, Text::new("\\ue056")).on_press(Message::Refresh);

        let header_line = Row::new().push(refresh_button).align_items(Alignment::Start);
        let display_name =
            Text::new(self.account.clone().unwrap_or_default().displayName).width(Length::Fill);

        let row1_left = Row::new()
            .push(Text::new("name").width(Length::Units(40)))
            .push(display_name);

        let row1_right = Row::new()
            .push(Text::new("id").width(Length::Units(40)))
            .push(Text::new("123456").width(Length::Fill));

        let col = Row::new()
            .push(row1_left.width(Length::Fill))
            .push(row1_right.width(Length::Fill))
            .align_items(Alignment::Center);

        

        let content = Column::new().push(header_line).push(col);
        Container::new(content).center_x().center_y().into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::WokerConnect(event) => match event {
                WorkEvent::Ready(s) => {
                    self.work_sender.sender = Some(s);
                }
                WorkEvent::WorkReturn(lcu_package) => {
                    println!("{:?}", lcu_package);
                    self.account = match lcu_package {
                        LCUpackage::Summoner(s) => Some(s),
                    };
                }
                WorkEvent::Finished => {}
            },
            Message::Refresh => {
                let r = await_sender(self.work_sender.clone(), WorkInput::Refresh);

                println!("send result:{:?}", r);
            }
            Message::SendMessage => {
                let r = await_sender(self.work_sender.clone(), WorkInput::SendMessage);
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.worker_list.iter().map(WorkMap::subscription))
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
            always_on_top: false,
            position: window::Position::Specific(5, 466),
            icon: None,
        },
        // flags: c,
        default_font: Some(include_bytes!("../方正准圆简体.ttf")),
        ..Settings::default()
    });
}

fn await_sender(
    work_sender: WorkerSender,
    input: WorkInput,
) -> Result<(), iced::futures::channel::mpsc::SendError> {
    let mut sender = work_sender.sender.unwrap();
    sender.start_send(input)
    // tokio::task::block_in_place(move || {
    //     tokio::runtime::Handle::current()
    //         .block_on(async move { sender.send(WorkInput::Refresh).await })
    // })
}
