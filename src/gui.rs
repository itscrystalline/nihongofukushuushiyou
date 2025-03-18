use std::path::Path;
use std::path::PathBuf;

use crate::libfukushuu::shitsumon::OptionPair;
use crate::libfukushuu::shitsumon::Question;
use crate::Error;
use eframe::egui;
use eframe::egui::Align;
use eframe::egui::FontData;
use eframe::egui::Frame;
use eframe::egui::ImageSource;
use eframe::egui::InnerResponse;
use eframe::egui::Label;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::epaint::text::FontInsert;
use eframe::epaint::text::InsertFontFamily;
use log::debug;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite::Result;

struct GuiState<'a> {
    conn: &'a Connection,
    questions: Vec<QuestionState>,
    question_count: u32,
    choices_count: u32,

    current_question: usize,
}
struct QuestionState {
    question: OptionPair,
    options: Vec<OptionPair>,
    correct_idx: usize,
}
impl QuestionState {
    fn from_questions(from: Vec<Question>) -> Vec<Self> {
        from.into_iter()
            .map(|q| {
                let (options, correct_idx) = q.get_options_randomize();
                Self {
                    question: q.front,
                    options,
                    correct_idx,
                }
            })
            .collect()
    }
}

impl<'a> GuiState<'a> {
    fn new(
        ctx: &eframe::CreationContext,
        conn: &'a Connection,
        questions: Vec<Question>,
        question_count: u32,
        choices_count: u32,
    ) -> Self {
        add_fonts(ctx);
        egui_extras::install_image_loaders(&ctx.egui_ctx);

        Self {
            conn,
            questions: QuestionState::from_questions(questions),
            question_count,
            choices_count,

            current_question: 0,
        }
    }

    fn draw_question_frame(&self, ui: &mut Ui, question_idx: u32) {
        let OptionPair(text, image) = &self.questions[question_idx as usize].question;
        let options = &self.questions[question_idx as usize].options;
        let mut results: Option<Vec<Response>> = None;

        ui.vertical(|ui| {
            if let Some(img) = image {
                if let Some(str) = img.as_os_str().to_str() {
                    ui.image(format!("file://{}", str));
                }
            }
            if let Some(text) = text {
                ui.label(RichText::new(text).size(40.0));
            }
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                _ = results.insert(
                    options
                        .iter()
                        .enumerate()
                        .map(|(idx, OptionPair(opt, img_path))| {
                            ui.scope_builder(
                                UiBuilder::new()
                                    .id_salt(format!("option_{idx}"))
                                    .sense(Sense::click()),
                                |ui| {
                                    let response = ui.response();
                                    let visuals = ui.style().interact(&response);
                                    let text_color = visuals.text_color();

                                    Frame::canvas(ui.style())
                                        .fill(visuals.bg_fill.gamma_multiply(0.3))
                                        .stroke(visuals.bg_stroke)
                                        .inner_margin(ui.spacing().menu_margin)
                                        .show(ui, |ui| {
                                            //ui.vertical_centered(|ui| {
                                            if let Some(opt) = opt {
                                                ui.add(
                                                    Label::new(
                                                        RichText::new(opt.to_string())
                                                            .color(text_color)
                                                            .size(32.0),
                                                    )
                                                    .selectable(false),
                                                );
                                            }
                                            //});
                                        });
                                },
                            )
                            .response
                        })
                        .collect(),
                );
            });
        });
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
    questions: Vec<Question>,
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
