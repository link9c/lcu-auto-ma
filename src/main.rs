#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod lcu;
mod style;
mod worker;

use iced::window::Icon;
// use anyhow::Result;
use lcu::entity::{LcuPackage, LcuResult, SummonerCollection};
use worker::{Message, WorkEvent, WorkInput, WorkMap, WorkerSender};
// use std::collections::HashMap;\
use iced::{
    alignment, executor, window, Application, Command, Element, Length, Settings, Subscription,
};

use iced::widget::{
    button, image::Handle, scrollable, svg, Button, Column, Container, Image, Row, Scrollable, Svg,
    Text,
};

// use lcu::winhook::loop_send_by_key;

static mut AUTO: bool = false;
#[derive(Default)]
pub struct MainUI {
    refresh_button: button::State,
    send_button: button::State,
    scrollable: scrollable::State,
    // auto_button: button::State,
    account: Option<SummonerCollection>,
    message: String,
    game_status: String,
    // api: ApiClient,
    worker_list: Vec<WorkMap>,
    work_sender: WorkerSender,
}

impl MainUI {
    fn new() -> MainUI {
        MainUI {
            refresh_button: button::State::new(),
            send_button: button::State::new(),
            // auto_button: button::State::new(),
            account: None,
            message: String::from(""),
            game_status: String::from("未连接"),

            // api: ApiClient::default(),
            worker_list: vec![WorkMap::new(0)],
            work_sender: WorkerSender::default(),
            scrollable: scrollable::State::new(),
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
        String::from("LOL赛马")
    }

    fn view(&mut self) -> Element<Message> {
        let summ = self.account.clone().unwrap_or_default();

        let s_avatar = Container::new(
            Image::new(Handle::from_memory(summ.avatar))
                .height(Length::Units(125))
                .width(Length::Units(125)),
        )
        .height(Length::Units(125))
        .width(Length::Units(125));

        let s_name = Text::new(format!("名字:{}", summ.summor.displayName));
        let s_level = Text::new(format!(
            "等级:{}({}/{})",
            summ.summor.summonerLevel, summ.summor.xpSinceLastLevel, summ.summor.xpUntilNextLevel
        ));

        let s_basic_info = Container::new(
            Row::new()
                .push(s_avatar)
                .push(
                    Column::new()
                        .push(s_name)
                        .push(s_level)
                        .spacing(3)
                        .width(Length::Fill),
                )
                .spacing(5),
        )
        .style(style::ContainerStyle)
        .padding(10);

        let log_text = Container::new(
            Scrollable::new(&mut self.scrollable)
                .push(
                    Container::new(Text::new(self.message.clone()))
                        .height(Length::Units(60))
                        .padding(10),
                )
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(style::ContainerStyle);

        let button_svg = Svg::new(svg::Handle::from_path(format!(
            "{}/resource/refresh.svg",
            env!("CARGO_MANIFEST_DIR")
        )))
        .width(Length::Units(30))
        .height(Length::Units(20));

        let refresh_button =
            Button::new(&mut self.refresh_button, button_svg).on_press(Message::Refresh);

        let send_button =
            Button::new(&mut self.send_button, Text::new("发送")).on_press(Message::SendMessage);
        let game_status = Container::new(
            Text::new(format!("状态:{}", self.game_status.clone()))
              
        
        ).height(Length::Units(30)).width(Length::Fill).align_y(alignment::Vertical::Center).align_x(alignment::Horizontal::Right);
        let button_group = Container::new(
            Row::new()
                .push(send_button)
                .push(refresh_button)
                .push(game_status),
        );

        let content = Column::new()
            .push(s_basic_info)
            .push(log_text)
            .push(button_group)
            .padding(5)
            .spacing(5);

        Container::new(content).center_x().center_y().into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::WokerConnect(event) => match *event {
                WorkEvent::Ready(s) => {
                    self.work_sender.sender = Some(s);
                }
                WorkEvent::WorkReturn(lcu_result) => {
                    // println!("{:?}", lcu_result);
                    match lcu_result {
                        LcuResult::Ok(pack) => match pack {
                            LcuPackage::Summoner(s) => {
                                self.game_status = s.status.clone();
                                self.account = Some(s);
                            }
                            LcuPackage::Message(s) => {
                                self.message = s.message;
                                self.game_status = s.status;
                            }
                        },
                        LcuResult::Err(err) => {
                            self.message = err.to_string();
                        }
                    };
                }
                WorkEvent::Finished => {}
            },
            Message::Refresh => {
                let _r = await_sender(self.work_sender.clone(), WorkInput::Refresh);
            }
            Message::SendMessage => {
                let _r = await_sender(self.work_sender.clone(), WorkInput::SendMessage);
            }
            Message::Auto => unsafe {
                AUTO = !AUTO;
            },
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.worker_list.iter().map(WorkMap::subscription))
    }
}

#[tokio::main]
async fn main() {
    // 监听键盘事件线程
    // let _ = std::thread::spawn(|| {
    //     loop_send_by_key();
    // });
    // 循环处理线程
    // let _ = std::thread::spawn(|| loop {
    //     std::thread::sleep(std::time::Duration::new(2, 0));
    //     unsafe {
    //         if AUTO {
    //             println!("looping");
    //         }
    //     }
    // });

    
       
         
    let icon = Icon::from_rgba(style::icon_raw::DATA.to_vec(), 32, 32).unwrap();
   
    let _ = MainUI::run(Settings {
        window: window::Settings {
            size: (320, 257),
            min_size: Some((200, 100)),
            max_size: None,
            resizable: false,
            decorations: true,
            transparent: false,
            always_on_top: false,
            position: window::Position::Specific(750, 366),
            icon: Some(icon),
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
}
