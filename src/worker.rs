// use iced::{futures::channel::mpsc, Subscription};
use crate::lcu::{self, Summoner};
// use anyhow::Result;
use iced_native::futures::channel::mpsc;
use iced_native::subscription::{self, Subscription};
#[derive(Debug, Clone)]
pub enum WorkEvent {
    Ready(mpsc::Sender<WorkInput>),
    WorkReturn(Summoner),
    Finished
    // Ready
}
#[derive(Debug, Clone)]
pub enum WorkInput {
    Pending,
    Refresh,
}
#[derive(Debug)]
pub enum WorkState {
    Starting,
    Ready(mpsc::Receiver<WorkInput>),
    Finished
    
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
}

pub fn factory(id: usize, state: WorkState) -> Subscription<WorkEvent> {
    subscription::unfold(id, state, |state| async move {
        match state {
            WorkState::Starting => {
                let (mut sender, receiver) = mpsc::channel(100);

                
                let _ = sender.start_send(WorkInput::Refresh);

                (Some(WorkEvent::Ready(sender)), WorkState::Ready(receiver))
            }
            WorkState::Ready(mut receiver) => {
                use iced_native::futures::StreamExt;

                let input = receiver.select_next_some().await;

                match input {
                    WorkInput::Refresh => {
                        println!("refresh");
                        let mut api = lcu::ApiClient::default();
                        api.init_client();

                        let summ = api.summoner().await.unwrap_or_default();
                        // println!("aa{:?}",summ);

                        (
                            Some(WorkEvent::WorkReturn(summ)),
                            WorkState::Ready(receiver),
                        )
                    }
                    WorkInput::Pending => { (
                        Some(WorkEvent::Finished),
                        WorkState::Finished,
                    )},
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

#[derive(Debug, Default,Clone)]
pub struct WorkerSender {
    pub sender: Option<mpsc::Sender<WorkInput>>,
}
