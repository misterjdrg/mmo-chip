use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{State, WebSocketUpgrade, ws::Message},
    response::Response,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum DieImportState {
    Peeked,
    ShotDecoding,
    ShotDecoded,
    TileWritten { current: u32, total: u32 },
    Done,
}
#[derive(Debug, Clone)]
pub enum RealtimeEvent {
    AnnotationChange { die_id: Uuid, new_revision: u32 },
    DieImportStateChange { die_id: Uuid, state: DieImportState },
    MLJobUpdate { die_id: Uuid, job: () },
}

impl RealtimeEvent {
    fn should_recv(&self, subs: &HashSet<Uuid>) -> bool {
        match self {
            Self::AnnotationChange {
                die_id,
                new_revision,
            } if subs.contains(die_id) => true,
            Self::MLJobUpdate { die_id, job } if subs.contains(die_id) => true,
            Self::DieImportStateChange { die_id, state } if subs.contains(die_id) => true,
            _ => false,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum WsMessageRecv {
    Ping,
    #[serde(rename_all = "camelCase")]
    Subscribe {
        die_id: Uuid,
    },
    #[serde(rename_all = "camelCase")]
    Unsubscribe {
        die_id: Uuid,
    },
}
#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum WsMessageSend {
    Pong,
    #[serde(rename_all = "camelCase")]
    Annotations {
        die_id: Uuid,
        rev: u32,
    },
    #[serde(rename_all = "camelCase")]
    MLJob {
        die_id: Uuid,
        job: (),
    },
    ImportState {
        die_id: Uuid,
        state: DieImportState,
    },
}

pub async fn websocket(state: State<Arc<crate::State>>, ws: WebSocketUpgrade) -> Response {
    let mut rx = state.rt_sender.subscribe();
    let mut dies = HashSet::<Uuid>::new();
    ws.on_upgrade(async move |mut ws| {
        loop {
            tokio::select! {
                r = rx.recv() => {
                    let Ok(r) = r else {
                        // should never happen
                        // but if happens nothing to do just exit
                        break;
                    };

                    if !r.should_recv(&dies) {
                        continue;
                    }

                    match r {
                        RealtimeEvent::AnnotationChange { die_id, new_revision } => {
                            if ws.send(Message::text(serde_json::to_string(&WsMessageSend::Annotations { die_id, rev: new_revision }).unwrap())).await.is_err() {
                                break;
                            }
                        }
                        RealtimeEvent::MLJobUpdate { die_id, job } => {
                            if ws.send(Message::text(serde_json::to_string(&WsMessageSend::MLJob { die_id, job }).unwrap())).await.is_err() {
                                break;
                            }
                        }
                        RealtimeEvent::DieImportStateChange { die_id, state } => {
                            if ws.send(Message::text(serde_json::to_string(&WsMessageSend::ImportState { die_id, state }).unwrap())).await.is_err() {
                                break;
                            }
                        }

                    }
                }
                msg = ws.recv() => {
                    let Some(msg) = msg else {
                        log::info!("channel closed");
                        break;
                    };
                    let Ok(msg) = msg else {
                        log::info!("mesg recv error");
                        continue;
                    };
                    let Message::Text(text) = msg else {
                        log::info!("mesg type not text");
                        continue;
                    };
                    let Ok(msg) = serde_json::from_str::<WsMessageRecv>(&text) else {
                        log::info!("cant parse msg: {text}");
                        continue;
                    };

                    match msg {
                        WsMessageRecv::Ping => {
                            if ws.send(Message::text(serde_json::to_string(&WsMessageSend::Pong).unwrap())).await.is_err() {
                                break;
                            }
                        }
                        WsMessageRecv::Subscribe { die_id } => {
                            dies.insert(die_id);
                        }
                        WsMessageRecv::Unsubscribe { die_id } => {
                            dies.remove(&die_id);
                        }
                    }
                }
            }
        }
    })
}

pub async fn rt_to_console(state: Arc<crate::State>) {
    let mut rt = state.rt_sender.subscribe();

    while let Ok(event) = rt.recv().await {
        log::info!("realtime: {event:?}");
    }
}
