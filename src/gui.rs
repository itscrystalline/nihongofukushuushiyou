
use crate::libfukushuu::shitsumon::OptionPair;
use crate::libfukushuu::shitsumon::Question;
use crate::Error;
use eframe::egui;
use eframe::egui::Align;
use eframe::egui::FontData;
use eframe::egui::Frame;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::epaint::text::FontInsert;
use eframe::epaint::text::InsertFontFamily;
use log::debug;
use rusqlite::Connection;
use rusqlite::Result;

struct GuiState<'a> {
    conn: &'a Connection,
    questions: &'a mut [Question],
    question_count: u32,
    choices_count: u32,

    current_question: usize,
}

impl<'a> GuiState<'a> {
    fn new(
        ctx: &eframe::CreationContext,
        conn: &'a Connection,
        questions: &'a mut [Question],
        question_count: u32,
        choices_count: u32,
    ) -> Self {
        add_fonts(ctx);

        Self {
            conn,
            questions,
            question_count,
            choices_count,

            current_question: 0,
        }
    }

    fn draw_question_frame(&self, ui: &mut Ui, question_idx: u32) {
        let OptionPair(text, image) = &self.questions[question_idx as usize].front;
        if let Some(text) = text {
            ui.label(RichText::new(text).size(40.0));
        }
    }
}

fn add_fonts(ctx: &eframe::CreationContext) {
    ctx.egui_ctx.add_font(FontInsert::new(
        "Noto Sans JP",
        FontData::from_static(include_bytes!("./fonts/NotoSansJP.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
    ctx.egui_ctx.add_font(FontInsert::new(
        "Inter",
        FontData::from_static(include_bytes!("./fonts/Inter.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
}

impl eframe::App for GuiState<'_> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let scroll = ScrollArea::horizontal().auto_shrink(false);
        let mut scroll_to = None;

        egui::TopBottomPanel::bottom("question_dots").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for idx in 0..self.questions.len() {
                    if ui.button(format!("{}", idx + 1)).clicked() {
                        debug!("current: {idx}");
                        self.current_question = idx;
                        scroll_to = Some(self.current_question)
                    }
                }
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            scroll.show(ui, |ui| {
                let height = ui.available_height();
                let width = ui.available_width();
                ui.horizontal(|ui| {
                    for idx in 0..self.question_count {
                        let resp = ui
                            .scope_builder(UiBuilder::new().id_salt(format!("q_{}", idx)), |ui| {
                                Frame::default().show(ui, |ui| {
                                    ui.set_width(width);
                                    ui.set_height(height);

                                    self.draw_question_frame(ui, idx);
                                })
                            })
                            .response;
                        if let Some(scroll_dest) = scroll_to {
                            if scroll_dest == idx as usize {
                                resp.scroll_to_me(Some(Align::Min));
                                scroll_to = None;
                            }
                        }
                    }
                });
            });
        });
    }
}

pub fn init_gui(
    conn: &Connection,
    questions: &mut [Question],
    question_count: u32,
    choices_count: u32,
) -> Result<(), Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "日本語復習しよう!",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(GuiState::new(
                cc,
                conn,
                questions,
                question_count,
                choices_count,
            )))
        }),
    )?;

    Ok(())
}
