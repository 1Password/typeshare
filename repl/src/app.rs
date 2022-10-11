use eframe::{App, Frame};
use egui::{Align, Context, Layout, Ui};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxSet};
use syntect::util::LinesWithEndings;
use typeshare_core::language::Language;

const SWIFT_SYNTAX: &str = include_str!("../syntaxes/Swift.sublime-syntax");

pub struct TypeshareReplApp<Typescript, Swift, Kotlin, Go> {
    user_source: String,
    output: String,
    selected_language: LanguageSelector,

    typescript: Typescript,
    swift: Swift,
    kotlin: Kotlin,
    go: Go,

    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
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
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

fn syntax_highlighter(
    syntax_set: &SyntaxSet,
    theme: &Theme,
    extension: &str,
    ui: &Ui,
    text: &str,
) -> egui::text::LayoutJob {
    let syntax = syntax_set.find_syntax_by_extension(extension).unwrap();

    let mut job = egui::text::LayoutJob::default();
    let mut h = HighlightLines::new(syntax, theme);

    for line in LinesWithEndings::from(text) {
        let ranges = h.highlight_line(line, syntax_set).unwrap();
        for v in ranges {
            let front = v.0.foreground;
            job.append(
                v.1,
                0.0,
                egui::TextFormat::simple(
                    egui::TextStyle::Monospace.resolve(ui.style()),
                    egui::Color32::from_rgb(front.r, front.g, front.b),
                ),
            );
        }
    }
    job
}
fn lang_extension(lang: &LanguageSelector) -> &str {
    match lang {
        LanguageSelector::Kotlin => "java",
        LanguageSelector::Go => "go",
        LanguageSelector::Swift => "swift",
        LanguageSelector::Typescript => "js",
    }
}

fn current_theme(ui: &Ui) -> &str {
    if ui.style().visuals.dark_mode {
        "base16-ocean.dark"
    } else {
        "base16-ocean.light"
    }
}

#[derive(Copy, Clone, PartialEq)]
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
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let old_user_source = self.user_source.clone();
        let old_lang_selected = self.selected_language.clone();
        let Self {
            ref mut user_source,
            ref mut selected_language,
            ref mut output,
            ref syntax_set,
            ref theme_set,
            ref kotlin,
            ref go,
            ref typescript,
            ref swift,
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            let theme = &theme_set.themes[current_theme(ui)];
            let mut rust_layout = |ui: &Ui, text: &str, wrap_width: f32| {
                let mut job = syntax_highlighter(syntax_set, theme, "rs", ui, text);
                job.wrap.max_width = wrap_width;
                ui.fonts().layout_job(job)
            };
            let id = ui.make_persistent_id("top_header");
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, |ui| {
                    ui.heading("Typeshare REPL");
                })
                .body(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::LEFT), |ui| {
                        ui.selectable_value(
                            selected_language,
                            LanguageSelector::Typescript,
                            "Typescript",
                        );
                        ui.selectable_value(selected_language, LanguageSelector::Swift, "Swift");
                        ui.selectable_value(selected_language, LanguageSelector::Kotlin, "Kotlin");
                        ui.selectable_value(selected_language, LanguageSelector::Go, "Go");
                    });
                });
            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.add_sized(
                        (ui.available_width() / 2.0, ui.available_height()),
                        egui::TextEdit::multiline(user_source)
                            .code_editor()
                            .layouter(&mut rust_layout),
                    );
                });
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    let mut output_layout = |ui: &Ui, text: &str, wrap_width: f32| {
                        let mut job = syntax_highlighter(
                            syntax_set,
                            theme,
                            lang_extension(&old_lang_selected),
                            ui,
                            text,
                        );
                        job.wrap.max_width = wrap_width;
                        ui.fonts().layout_job(job)
                    };
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(output)
                            .code_editor()
                            .interactive(false)
                            .layouter(&mut output_layout),
                    );
                })
            })
        });
        if old_user_source != *user_source || old_lang_selected != *selected_language {
            let mut out: Vec<u8> = Vec::new();
            let _ = match *selected_language {
                LanguageSelector::Kotlin => {
                    typeshare_core::process_input(user_source, kotlin, &mut out)
                }
                LanguageSelector::Go => typeshare_core::process_input(user_source, go, &mut out),
                LanguageSelector::Typescript => {
                    typeshare_core::process_input(user_source, typescript, &mut out)
                }
                LanguageSelector::Swift => {
                    typeshare_core::process_input(user_source, swift, &mut out)
                }
            };
            *output = String::from_utf8(out).unwrap();
        }
    }
}
