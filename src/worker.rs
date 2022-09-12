use std::collections::HashMap;
use std::sync::Arc;

use crate::lcu::api::ApiClient;
// use iced::{futures::channel::mpsc, Subscription};
use crate::lcu::entity::{
    GameSession, HorseInfo, LcuError, LcuPackage, LcuResult, MessageCollection, SendInfo,
    SummonerCollection,
};

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
    static ref CHATOOM: Arc<Mutex<GameSession>> = Arc::new(Mutex::new(GameSession::default()));
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
    WokerConnect(Box<WorkEvent>),
    Refresh,
    SendMessage,
    Auto,
}

pub fn factory(id: usize, state: WorkState) -> Subscription<Box<WorkEvent>> {
    subscription::unfold(id, state, |state| async move {
        match state {
            WorkState::Starting => {
                let (mut sender, receiver) = mpsc::channel(100);

                let _ = sender.start_send(WorkInput::Init);

                (
                    Some(Box::new(WorkEvent::Ready(sender))),
                    WorkState::Ready(receiver),
                )
            }

            WorkState::Ready(mut receiver) => {
                use iced_native::futures::StreamExt;

                let input = receiver.select_next_some().await;

                match input {
                    WorkInput::Init => {
                        let global_api_client = API.lock().await;
                        let res = get_current_summoner(global_api_client).await;
                        (
                            Some(Box::new(WorkEvent::WorkReturn(res))),
                            WorkState::Ready(receiver),
                        )
                    }

                    WorkInput::Pending => {
                        (Some(Box::new(WorkEvent::Finished)), WorkState::Finished)
                    }
                    WorkInput::Refresh => {
                        println!("refresh");
                        let mut api = ApiClient::default();
                        api.init_client();

                        let mut global_api_client = API.lock().await;
                        *global_api_client = api;

                        let res = get_current_summoner(global_api_client).await;

                        (
                            Some(Box::new(WorkEvent::WorkReturn(res))),
                            WorkState::Ready(receiver),
                        )
                    }
                    WorkInput::SendMessage => {
                        let global_api_client = API.lock().await;
                        let res = get_horse_rank_in_select_room(global_api_client).await;

                        (
                            Some(Box::new(WorkEvent::WorkReturn(res))),
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
            Ok(v) => {
                let avatar = api
                    .clone()
                    .get_avatar(v.profileIconId.to_string().as_str())
                    .await;

                let status = api.clone().get_gameflow_phase().await;

                LcuResult::Ok(LcuPackage::Summoner(SummonerCollection {
                    summor: v,
                    status: status.unwrap(),
                    avatar: avatar.unwrap(),
                }))
            }
            Err(e) => {
                println!("{:#}", e);
                LcuResult::Err(LcuError::JsonParseFailed)
            }
        }
    }
}

///获取战绩排名
///
///
async fn get_horse_rank_in_select_room(api: MutexGuard<'_, ApiClient>) -> LcuResult {
    let res = api.clone().get_session().await;

    if let Ok(game_session) = res {
        println!("{:?}", game_session);
        let users = game_session.myTeam;
        let side_uses = game_session.theirTeam;
        println!("other team{:?}", side_uses);
        let mut horse_room: Vec<HorseInfo> = Vec::new();
        for user in users {
            let mut hero_count: HashMap<u32, u8> = HashMap::new();
            let mut horse_info = HorseInfo::default();
            let user_result = api
                .clone()
                .get_summoners(user.summonerId.to_string().as_str())
                .await;
            if let Ok(user_info) = user_result {
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
                let hero_id = hero[0].0;

                let hero_name = api.clone().get_hero_name(*hero_id).await.unwrap();
                horse_info.favhero = hero_name.get("name").unwrap().to_string().replace('\"', "");
                horse_info.user = user_info.displayName;
                horse_info.summonerId = user_info.summonerId;
                horse_room.push(horse_info);
            } else {
                println!("error:{:?}", user_result);
            }
        }

        horse_room.sort_by(|x, y| y.cmp(x));
        let length = horse_room.len();
        let chat_room = game_session
            .chatDetails
            .chatRoomName
            .split('@')
            .collect::<Vec<&str>>();
        // let horse_ids = horse_room
        //     .iter()
        //     .map(|x| x.summonerId)
        // .collect::<Vec<u64>>();
        for (i, horse) in horse_room.iter().enumerate() {
            let rank = if i == 0 {
                "上等马"
            } else if i == length - 1 {
                "下等马"
            } else {
                "普通马"
            };

            let text = format!("{}--{}", rank, horse.text());

            let body = SendInfo {
                body: text.clone(),
                fromId: horse.summonerId.to_string(),
                fromSummonerId: horse.summonerId.to_string(),
                ..Default::default()
            };

            let res = api.clone().send_message(chat_room[0], body).await;
            println!("{:?}", res);

            // println!("--{:?}--{:?}", body, chat_room);

            // }
        }
        // let txt = horse_room
        //     .iter()
        //     .map(|x| x.text())
        //     .collect::<Vec<String>>()
        //     .join("\n");

        let status = api
            .clone()
            .get_gameflow_phase()
            .await
            .unwrap_or(String::from("disconnect"));

        LcuResult::Ok(LcuPackage::Message(MessageCollection {
            message: String::from("发送成功"),
            status: status,
        }))
    } else {
        LcuResult::Err(LcuError::ChampionSelectSessionErrror)
    }
}
