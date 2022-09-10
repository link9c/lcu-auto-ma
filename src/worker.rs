use std::collections::HashMap;
use std::sync::Arc;

use crate::lcu::api::ApiClient;
// use iced::{futures::channel::mpsc, Subscription};
use crate::lcu::entity::{GameSession, HorseInfo, LcuError, LcuPackage, LcuResult, SendInfo};

use iced_native::futures::channel::mpsc;
use iced_native::subscription::{self, Subscription};
use lazy_static::lazy_static;
use tokio::sync::{Mutex, MutexGuard};

lazy_static! {
    static ref API: Arc<Mutex<ApiClient>> = Arc::new(Mutex::new({
        let mut ac = ApiClient::default();
        ac.init_client();
        ac
    }));
    static ref CHATOOM: Arc<Mutex<GameSession>> = Arc::new(Mutex::new({
        GameSession::default()
       
    }));
}
#[derive(Debug, Clone)]
pub enum WorkEvent {
    Ready(mpsc::Sender<WorkInput>),
    WorkReturn(LcuResult),
    Finished, // Ready
}
#[derive(Debug, Clone)]
pub enum WorkInput {
    Pending,
    Refresh,
    SendMessage,
    Init,
}
#[derive(Debug)]
pub enum WorkState {
    Starting,
    Ready(mpsc::Receiver<WorkInput>),
    Finished,
}

pub struct WorkMap {
    id: usize,
}

impl WorkMap {
    pub fn new(id: usize) -> Self {
        WorkMap { id }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        factory(self.id, WorkState::Starting).map(Message::WokerConnect)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    WokerConnect(WorkEvent),
    Refresh,
    SendMessage,
    Auto,
}

pub fn factory(id: usize, state: WorkState) -> Subscription<WorkEvent> {
    subscription::unfold(id, state, |state| async move {
        match state {
            WorkState::Starting => {
                let (mut sender, receiver) = mpsc::channel(100);

                let _ = sender.start_send(WorkInput::Init);

                (Some(WorkEvent::Ready(sender)), WorkState::Ready(receiver))
            }
            WorkState::Ready(mut receiver) => {
                use iced_native::futures::StreamExt;

                let input = receiver.select_next_some().await;

                match input {
                    WorkInput::Refresh => {
                        println!("refresh");
                        let mut api = ApiClient::default();
                        api.init_client();

                        let mut global_api_client = API.lock().await;
                        *global_api_client = api;

                        let summ = get_current_summoner(global_api_client).await;

                        (
                            Some(WorkEvent::WorkReturn(summ)),
                            WorkState::Ready(receiver),
                        )
                    }
                    WorkInput::Pending => (Some(WorkEvent::Finished), WorkState::Finished),
                    WorkInput::SendMessage => {
                        let global_api_client = API.lock().await;
                        get_horse_rank_in_select_room(global_api_client).await;

                        (
                            Some(WorkEvent::WorkReturn(LcuResult::Ok(LcuPackage::Status(
                                "ss".to_string(),
                            )))),
                            WorkState::Ready(receiver),
                        )
                    }
                    WorkInput::Init => {
                        let global_api_client = API.lock().await;
                        let summ = get_current_summoner(global_api_client).await;
                        (
                            Some(WorkEvent::WorkReturn(summ)),
                            WorkState::Ready(receiver),
                        )
                    }
                }
            }
            WorkState::Finished => {
                // We do not let the stream die, as it would start a
                // new download repeatedly if the user is not careful
                // in case of errors.
                iced::futures::future::pending().await
            }
        }
    })
}

#[derive(Debug, Default, Clone)]
pub struct WorkerSender {
    pub sender: Option<mpsc::Sender<WorkInput>>,
}

async fn get_current_summoner(api: MutexGuard<'_, ApiClient>) -> LcuResult {
    if let Some(e) = api.init_error.clone() {
        LcuResult::Err(e)
    } else {
        let lcu_result = api.clone().get_current_summoner().await;

        match lcu_result {
            Ok(v) => LcuResult::Ok(LcuPackage::Summoner(v)),
            Err(e) => {
                println!("{:#}", e);
                LcuResult::Err(LcuError::JsonParseFailed)
            }
        }
    }
}

async fn get_game_flow(api: MutexGuard<'_, ApiClient>) -> LcuResult {
    if let Some(e) = api.init_error.clone() {
        LcuResult::Err(e)
    } else {
        let lcu_result = api.clone().get_gameflow_phase().await;

        match lcu_result {
            Ok(v) => {
                let status = v.to_string().replace('\"', "");
                if status == "None" {
                    LcuResult::Ok(LcuPackage::Status(String::from("Room")))
                } else {
                    LcuResult::Ok(LcuPackage::Status(status))
                }
            }
            Err(e) => {
                println!("{:#}", e);
                LcuResult::Err(LcuError::JsonParseFailed)
            }
        }
    }
}

async fn get_horse_rank_in_select_room(api: MutexGuard<'_, ApiClient>) {
    let res = api.clone().get_session().await;

    if let Ok(game_session) = res {
        println!("{:?}", game_session);
        let users = game_session.myTeam;
        let mut horse_room: Vec<HorseInfo> = Vec::new();
        for user in users {
            let mut hero_count: HashMap<u32, u8> = HashMap::new();
            let mut horse_info = HorseInfo::default();
            if let Ok(user_info) = api
                .clone()
                .get_summoners(user.summonerId.to_string().as_str())
                .await
            {
                for index in 1..3_u32 {
                    if let Ok(matchs_history) =
                        api.clone().get_match_history(&user_info.puuid, index).await
                    {
                        for game in &matchs_history.games.games {
                            if game.queueId != 830 && game.queueId != 840 && game.queueId != 850 {
                                let pt = &game.participants[0];
                                let hero = pt.championId;
                               

                                hero_count
                                    .entry(hero)
                                    .and_modify(|counter| *counter += 1)
                                    .or_insert(1);

                                horse_info.deaths += pt.stats.deaths;
                                horse_info.kills += pt.stats.kills;
                                horse_info.assists += pt.stats.assists;
                                if pt.stats.win {
                                    horse_info.win += 1;
                                } else {
                                    horse_info.defeat += 1;
                                }
                            }
                        }
                    }
                }
                println!("{:?}", hero_count);
                // 获取最常用英雄
                let mut hero = hero_count.iter().collect::<Vec<(&u32, &u8)>>();
                hero.sort_by(|a, b| b.1.cmp(a.1));
                let hero_id = hero[0].1;

                let hero_name = api.clone().get_hero_name(*hero_id).await.unwrap();
                horse_info.favhero = hero_name.get("name").unwrap().to_string().replace('\"', "");
                horse_info.user = user_info.displayName;
                horse_info.summonerId = user_info.summonerId;
                horse_room.push(horse_info);
            }
        }

        horse_room.sort_by_key(|x| x.win_rate().ceil() as u32);
        let length = horse_room.len();
        let chat_room = game_session
            .chatDetails
            .chatRoomName
            .split('@')
            .collect::<Vec<&str>>();
        for (i, horse) in horse_room.iter().enumerate() {
            let mut rank = "上等马";
            if i == 0 {
                rank = "下等马"
            } else if (1..length - 2).contains(&i) {
                rank = "中等马"
            }
            let mut body = SendInfo::default();
            let summ = api.clone().get_current_summoner().await.unwrap();
            body.fromId = summ.summonerId.to_string();
            body.fromSummonerId = horse.summonerId.to_string();
            body.body = format!("{}--{}", rank, horse.text());
            println!("--{:?}--{:?}", body, chat_room);
            let res = api.clone().send_message(chat_room[0], body).await;
            println!("{:?}", res);
        }
    }
}
