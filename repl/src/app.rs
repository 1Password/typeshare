use eframe::emath::Vec2;
use eframe::epaint::Rgba;
use eframe::{App, Frame, Storage};
use egui::panel::Side;
use egui::{Align, Context, Layout, Ui, Visuals};
use std::time::Duration;
use syntect::parsing::SyntaxSet;
use typeshare_core::language::Language;

pub struct TypeshareReplApp<Typescript, Swift, Kotlin, Go> {
    user_source: String,
    output: String,
    selected_language: LanguageSelector,

    typescript: Typescript,
    swift: Swift,
    kotlin: Kotlin,
    go: Go,

    syntax_set: SyntaxSet,
}

impl<T, S, K, G> TypeshareReplApp<T, S, K, G>
where
    T: Language,
    S: Language,
    K: Language,
    G: Language,
{
    pub fn new(typescript: T, swift: S, kotlin: K, go: G) -> Self {
        let mut ss = SyntaxSet::load_defaults_newlines().into_builder();
        ss.add(SyntaxDefinition::load_from_str(SWIFT_SYNTAX, true, Some("swift")).unwrap());
        let ss = ss.build();
        Self {
            user_source: String::default(),
            output: String::default(),
            selected_language: LanguageSelector::Typescript,
            typescript,
            swift,
            kotlin,
            go,
            syntax_set: ss,
        }
    }
}

#[derive(PartialEq)]
enum LanguageSelector {
    Typescript,
    Swift,
    Kotlin,
    Go,
}

impl<T, S, K, G> App for TypeshareReplApp<T, S, K, G>
where
    T: Language,
    S: Language,
    K: Language,
    G: Language,
{
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let old_user_source = self.user_source.clone();
        let Self {
            ref mut user_source,
            ref mut selected_language,
            ref mut output,
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Typescript REPL");
            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.add_sized(
                        (ui.available_width() / 2.0, ui.available_height()),
                        egui::TextEdit::multiline(user_source).code_editor(),
                    );
                });
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        ui.selectable_value(
                            selected_language,
                            LanguageSelector::Typescript,
                            "Typescript",
                        );
                        ui.selectable_value(selected_language, LanguageSelector::Swift, "Swift");
                        ui.selectable_value(selected_language, LanguageSelector::Kotlin, "Kotlin");
                        ui.selectable_value(selected_language, LanguageSelector::Go, "Go");
                    });
                    ui.separator();
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(output)
                            .code_editor()
                            .interactive(false),
                    );
                })
            })
        });
    }
}
