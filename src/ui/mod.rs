use std::sync::atomic::Ordering;

use eframe::egui::{Context, Ui};
use eframe::{egui, App, Frame};

use crate::ui::play::Play;

pub mod play;

pub trait View {
    fn ui(&mut self, ui: &mut Ui);
}

impl App for Play {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| self.ui(ui));
        egui::Window::new("音轨")
            .open(&mut self.tracks_enable)
            .show(ctx, |ui| {
                for (enable, index) in self.midi.track_num.lock().unwrap().iter_mut() {
                    if ui.checkbox(enable, format!("Track {}", index)).changed() {
                        self.notify_merge = true;
                    }
                }
                if self.notify_merge {
                    let range = self
                        .midi
                        .track_num
                        .lock()
                        .unwrap()
                        .iter()
                        .filter_map(|(enable, index)| if *enable { Some(*index) } else { None })
                        .collect::<Vec<_>>();
                    self.midi.merge_tracks(&range);
                    self.midi
                        .hit_rate
                        .store(self.midi.detect(self.offset), Ordering::Relaxed);
                    self.notify_merge = false;
                }
            });
    }
}