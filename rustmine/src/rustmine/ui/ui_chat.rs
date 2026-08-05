use crate::engine::ui::ui_factory::UiComponents;
use crate::rustmine::generation::world::World;
use egui::{Align2, Color32, vec2};
use std::time::{Duration, Instant};

const LIFETIME: Duration = Duration::from_secs(10);
const FADE_DURATION: Duration = Duration::from_secs(2);
const MAX_VISIBLE: usize = 10;
const HISTORY_SIZE: usize = 200;

pub enum ChatMessage {
    User(UserMessage),
    System(SystemMessage),
}

impl ChatMessage {
    fn created_at(&self) -> Instant {
        match self {
            ChatMessage::User(m) => m.created_at,
            ChatMessage::System(m) => m.created_at,
        }
    }

    fn display_text(&self, username: &str) -> String {
        match self {
            ChatMessage::User(m) => format!("<{}> {}", username, m.text),
            ChatMessage::System(m) => m.text.clone(),
        }
    }
}

pub struct UserMessage {
    text: String,
    created_at: Instant,
}

impl UserMessage {
    fn new(text: String) -> Self {
        Self {
            text,
            created_at: Instant::now(),
        }
    }
}

pub struct SystemMessage {
    text: String,
    created_at: Instant,
}

impl SystemMessage {
    fn new(text: String) -> Self {
        Self {
            text,
            created_at: Instant::now(),
        }
    }
}

pub(crate) struct ChatUI {
    pub world_name: String,
    pub username: String,
    pub messages: Vec<ChatMessage>,
    pub new_message: String,
    pub pending_teleport: Option<[f32; 3]>,
}

impl ChatUI {
    pub fn new(username: String, world_name: String) -> Self {
        Self {
            username,
            world_name,
            messages: Vec::new(),
            new_message: String::new(),
            pending_teleport: None,
        }
    }

    pub fn retain_message(&mut self) {
        if self.messages.len() >= HISTORY_SIZE {
            self.messages.remove(0);
        }
    }

    pub fn render_chat_closed(&mut self, ctx: &egui::Context, uic: &mut UiComponents) {
        self.retain_message();

        let start = self.messages.len().saturating_sub(MAX_VISIBLE);

        egui::Area::new(egui::Id::new("chat"))
            .anchor(Align2::LEFT_BOTTOM, vec2(12.0, 0.0))
            .interactable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for message in &self.messages[start..] {
                        let alpha = fade_alpha(message);
                        if alpha <= 0.0 {
                            continue;
                        }

                        let msg = message.display_text(&self.username);
                        let color = Color32::from_white_alpha((alpha * 255.0) as u8);

                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(msg).size(uic.pt(40)).color(color),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    }
                });
            });
        ctx.request_repaint();
    }

    pub fn render_chat_opened(&mut self, ctx: &egui::Context, uic: &mut UiComponents) {
        self.retain_message();

        egui::Area::new(egui::Id::new("chat-opened"))
            .anchor(Align2::LEFT_BOTTOM, vec2(12.0, 0.0))
            .interactable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for message in self.messages.iter() {
                            let msg = message.display_text(&self.username);
                            uic.label(ui, &msg, uic.pt(40));
                        }
                    });
                    let response =
                        uic.input(ui, uic.vw(30), uic.vh(5), Some(256), &mut self.new_message);
                    response.request_focus();
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if enter_pressed && !self.new_message.trim().is_empty() {
                        let text = std::mem::take(&mut self.new_message);
                        if text.chars().nth(0).unwrap() != '/' {
                            self.messages
                                .push(ChatMessage::User(UserMessage::new(text)));
                        } else {
                            self.handle_commands(text.clone());
                        }
                    }
                });
            });
    }

    pub fn handle_commands(&mut self, command: String) {
        let tokens: Vec<&str> = command.split_whitespace().collect();

        if tokens.is_empty() {
            return;
        }

        match tokens[0] {
            "/teleport" | "/tp" => {
                if let Some(coords) = self.handle_teleport(&tokens[1..]) {
                    self.pending_teleport = Some(coords);
                    self.messages.push(ChatMessage::System(SystemMessage::new(
                        "Teleporting...".to_string(),
                    )));
                }
            }
            "/seed" => {
                let seed = World::load_world_settings(&self.world_name)
                    .unwrap()
                    .seed
                    .to_string();

                self.messages
                    .push(ChatMessage::System(SystemMessage::new(format!(
                        "Seed: {}",
                        seed
                    ))));
            }
            _ => {
                self.messages
                    .push(ChatMessage::System(SystemMessage::new(format!(
                        "Unknown command: {}",
                        tokens[0]
                    ))));
            }
        }
    }

    pub fn handle_teleport(&mut self, tokens: &[&str]) -> Option<[f32; 3]> {
        if tokens.len() != 3 {
            self.messages.push(ChatMessage::System(SystemMessage::new(
                "Usage: /teleport <x> <y> <z> | /tp <x> <y> <z>".to_string(),
            )));
            return None;
        }
        let coords: Result<Vec<f32>, _> = tokens.iter().map(|s| s.parse::<f32>()).collect();

        match coords {
            Ok(xyz) => Some([xyz[0], xyz[1], xyz[2]]),
            Err(_) => {
                self.messages.push(ChatMessage::System(SystemMessage::new(
                    "Invalid coordinates. Expected numbers".to_string(),
                )));
                None
            }
        }
    }
}

// Helpers
fn fade_alpha(message: &ChatMessage) -> f32 {
    let elapsed = message.created_at().elapsed();

    if elapsed >= LIFETIME {
        0.0
    } else {
        let fade_start = LIFETIME.saturating_sub(FADE_DURATION);
        if elapsed <= fade_start {
            1.0
        } else {
            let t = (elapsed - fade_start).as_secs_f32() / FADE_DURATION.as_secs_f32();
            1.0 - t.clamp(0.0, 1.0)
        }
    }
}
