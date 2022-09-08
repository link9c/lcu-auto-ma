use std::sync::Arc;

use crate::lcu::api::ApiClient;
// use iced::{futures::channel::mpsc, Subscription};
use crate::lcu::entity::{LcuError, LcuPackage, LcuResult, Summoner};

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

                        let summ = get_summoner(global_api_client).await;

                        (
                            Some(WorkEvent::WorkReturn(summ)),
                            WorkState::Ready(receiver),
                        )
                    }
                    WorkInput::Pending => (Some(WorkEvent::Finished), WorkState::Finished),
                    WorkInput::SendMessage => {
                        let global_api_client = API.lock().await;
                        let gameflow = global_api_client.clone().get_gameflow_phase().await;
                        println!("{:?}", gameflow);
                        (Some(WorkEvent::Finished), WorkState::Ready(receiver))
                    }
                    WorkInput::Init => {
                        let global_api_client = API.lock().await;
                        let summ = get_summoner(global_api_client).await;
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

async fn get_summoner(api: MutexGuard<'_, ApiClient>) -> LcuResult {
    if let Some(e) = api.init_error.clone() {
        LcuResult::Err(e)
    } else {
        let lcu_result = api.clone().get_summoner().await;

        match lcu_result {
            Ok(v) => {
                if v.get("errorCode").is_none() {
                    let s = serde_json::from_value::<Summoner>(v).unwrap();
                    LcuResult::Ok(LcuPackage::Summoner(s))
                } else {
                    println!("{:#}", v);
                    LcuResult::Err(LcuError::NotFind)
                }
            }
            Err(e) => {
                println!("{:#}", e);
                LcuResult::Err(LcuError::JsonParseFailed)
            }
        }
    }
}
