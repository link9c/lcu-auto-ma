use std::collections::HashMap;
use std::ops::Sub;

use iced::futures::{lock::Mutex, SinkExt};
use iced::futures::channel::mpsc;
use iced::futures::StreamExt;
use iced_native::subscription;


#[derive(Debug, Clone)]
pub enum Event {
    Init,
    Looping,

    MessageToUI(Output),
}
// UI->task
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Input {
    Start,
    Change(u64),
    Txt(String),
}
//task->UI
#[derive(Debug, Clone)]
pub enum Output {
    Card(CardInfo),
}
#[allow(dead_code)]
#[derive(Debug)]
pub enum State {
    Init,
    Starting,
    Finished,
    ExternalTask,
    Looping(mpsc::Receiver<Output>),
    // Starting,
    Ready(mpsc::Receiver<Input>),
}
#[derive(Debug, Clone)]
pub struct CardInfo {
    pub id: String,
    pub cn_name: String,
    pub en_name: String,
    pub jp_name: String,
    pub attr: String,
    pub desc: String,
}

impl Default for CardInfo {
    fn default() -> Self {
        Self {
            id: String::from(""),
            cn_name: String::from("中文名称"),
            en_name: String::from("英文名称"),
            jp_name: String::from("日文名称"),
            attr: String::from("属性"),
            desc: String::from(" "),
        }
    }
}

// Just a little utility function
pub fn init(id: usize) -> iced::Subscription<(usize, Event)> {
    // subscription:::
    subscription::unfold(id, State::Init, move |state| loop_work(id, state))
}

async fn loop_work(id: usize, state: State) -> (Option<(usize, Event)>, State) {
    // subscription::unfold(
    //     std::any::TypeId::of::<LoopScreenshot>(),
    //     State::Starting,
    //     |state| async move {

    // let mut art_hash:Vec<Vec<String>>=Vec::new();
    // let mut cards:HashMap<String,Vec<String>>=HashMap::new();

    match state {
        State::Init => {
            // let start = std::time::Instant::now();
            let mut art_hash = ART_HASH.lock().await;
            *art_hash = init_art_hash().unwrap();
            //println!("0 init hash {:?}", start.elapsed());
            let mut cards = CARDS.lock().await;
            *cards = init_cards_info().unwrap();
            //println!("0 init hash and cards {:?}", start.elapsed());
            // (Some(Event::Init), State::Starting((art_hash,cards)))
            (Some((id, Event::Init)), State::ExternalTask)
        }

        State::ExternalTask => {
            // Read next input sent from `Application`
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            let (mut loop_sender, loop_receiver) = mpsc::channel(10000);
            tokio::task::spawn(async move {
                let mut count = 0;
                let mut time = 50;
                let art_hash = ART_HASH.lock().await;
                let cards = CARDS.lock().await;
                let mut old_cid = 0;
                loop {
                    let start = std::time::Instant::now();

                    let (cid, info, count_time) = final_card_info(&*cards, &*art_hash);
                    if old_cid != cid {
                        old_cid = cid;
                        if count >= 300 && (time == 75 || time == 330) && cid == 0 {
                            time = 330;
                            count = 0
                        } else if count >= 1 && (time == 1000 || time == 330) && cid != 0 {
                            time = 75;
                            count = 0;
                        }
                    } else if count >= 300 && time == 75 {
                        time = 330;
                        count = 0
                    }

                    let card = if info.is_empty() {
                        CardInfo::default()
                    } else {
                        CardInfo {
                            id: info[0].clone(),
                            cn_name: info[1].clone(),
                            en_name: info[2].clone(),
                            jp_name: info[3].clone(),
                            attr: info[5].clone(),
                            desc: info[4].clone(),
                        }
                    };
                    //println!("send: {:?}", info);
                    let dt = if info.is_empty() {
                        start.elapsed().as_millis() + time + (count_time * 300) as u128 + 500
                    } else {
                        125 + time
                    };
                    //println!("id{},time{},dt{},count{}", id, time, dt, count);

                    loop_sender.send(Output::Card(card)).await.unwrap();
                    tokio::time::sleep(tokio::time::Duration::from_millis(dt as u64)).await;
                    count += 1;
                }
            });
            (Some((id, Event::Looping)), State::Looping(loop_receiver))
        }

        State::Looping(mut loop_receiver) => {
            let input = loop_receiver.select_next_some().await;
            (
                Some((id, Event::MessageToUI(input))),
                State::Looping(loop_receiver),
            )
        }

        _ => {
            let _: () = iced::futures::future::pending().await;

            unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection(pub mpsc::Sender<Input>);
