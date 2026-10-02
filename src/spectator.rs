use axum::{
    extract::{ws::{WebSocketUpgrade, Message, WebSocket}, State, Path},
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use uuid::Uuid;
use crate::{protocol::*, game::*, config::*};

struct ReactionOverlay {
    emoji: String,
    x: f32,
    y: f32,
    ttl_ms: u32,
}

struct Spectator {
    id: Uuid,
    tx: mpsc::Sender<ServerMsg>,
}

struct Room {
    id: Uuid,
    game: Arc<Mutex<GameState>>,
    bag: Arc<Mutex<BagRandomizer>>,
    spectators: Vec<Spectator>,
    reactions: Vec<ReactionOverlay>,
}

type RoomRegistry = Arc<Mutex<Option<Room>>>;

pub async fn create_room(registry: &RoomRegistry) -> Uuid {
    let mut reg = registry.lock().await;
    let room_id = Uuid::new_v4();
    let room = Room {
        id: room_id,
        game: Arc::new(Mutex::new(GameState::new())),
        bag: Arc::new(Mutex::new(BagRandomizer::new())),
        spectators: Vec::new(),
        reactions: Vec::new(),
    };
    *reg = Some(room);
    room_id
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(requested_room_id): Path<Uuid>,
    State(registry): State<RoomRegistry>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, requested_room_id, registry))
}

async fn handle_socket(socket: WebSocket, requested_room_id: Uuid, registry: RoomRegistry) {
    let (mut sender, mut receiver) = socket.split();
    let your_id = Uuid::new_v4();

    let (tx, mut rx) = mpsc::channel::<ServerMsg>(WS_CHANNEL_CAP);
    let spectator_id = Uuid::new_v4();

    {
        let mut reg = registry.lock().await;
        if let Some(room) = reg.as_mut() {
            if room.id != requested_room_id {
                let _ = sender.send(Message::Close(None)).await;
                return;
            }
            room.spectators.push(Spectator { id: spectator_id, tx: tx.clone() });
        } else {
            let _ = sender.send(Message::Close(None)).await;
            return;
        }
    }

    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap_or_default();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let welcome = ServerMsg::Welcome { room_id: requested_room_id, your_id };
    let _ = tx.send(welcome).await;

    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                            handle_client_msg(&registry, client_msg, your_id).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    None => break,
                    _ => {}
                }
            }
        }
    }

    sender_task.abort();

    let mut reg = registry.lock().await;
    if let Some(room) = reg.as_mut() {
        room.spectators.retain(|s| s.id != spectator_id);
    }
}

async fn handle_client_msg(registry: &RoomRegistry, msg: ClientMsg, your_id: Uuid) {
    let mut reg = registry.lock().await;
    if let Some(room) = reg.as_mut() {
        match msg {
            ClientMsg::Chat { text } => {
                let text = text.chars().take(MAX_CHAT_LENGTH).collect::<String>();
                let chat_msg = ServerMsg::Chat {
                    from: format!("Spectator#{}", your_id.simple().to_string().chars().take(4).collect::<String>()),
                    text,
                };
                room.spectators.retain(|s| s.tx.try_send(chat_msg.clone()).is_ok());
            }
            ClientMsg::Reaction { emoji } => {
                if REACTION_EMOJIS.contains(&emoji.as_str()) {
                    let x = rand::random::<f32>() * 10.0;
                    let y = 19.0;
                    let reaction = ReactionOverlay {
                        emoji: emoji.clone(),
                        x,
                        y,
                        ttl_ms: REACTION_TTL_MS,
                    };
                    room.reactions.push(reaction);
                    let react_msg = ServerMsg::Reaction { emoji, x, y };
                    room.spectators.retain(|s| s.tx.try_send(react_msg.clone()).is_ok());
                }
            }
        }
    }
}

pub async fn broadcast_task(registry: RoomRegistry) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(BROADCAST_INTERVAL_MS as u64));
    loop {
        interval.tick().await;
        let mut reg = registry.lock().await;
        if let Some(room) = reg.as_mut() {
            let snapshot: GameStateSnapshot = {
                let game = room.game.lock().await;
                (&*game).into()
            };
            room.reactions.retain_mut(|r| {
                r.ttl_ms = r.ttl_ms.saturating_sub(BROADCAST_INTERVAL_MS);
                r.ttl_ms > 0
            });
            let state_msg = ServerMsg::State(snapshot);
            room.spectators.retain(|s| s.tx.try_send(state_msg.clone()).is_ok());
            for r in &room.reactions {
                let react_msg = ServerMsg::Reaction { emoji: r.emoji.clone(), x: r.x, y: r.y };
                room.spectators.retain(|s| s.tx.try_send(react_msg.clone()).is_ok());
            }
        }
    }
}

pub fn get_room_game(registry: &RoomRegistry) -> Option<(Arc<Mutex<GameState>>, Arc<Mutex<BagRandomizer>>)> {
    let reg = registry.try_lock().ok()?;
    reg.as_ref().map(|r| (r.game.clone(), r.bag.clone()))
}