use super::*;
use crate::analyzer::Analyzer;

#[derive(Debug, Clone)]
pub struct PapersEditor {
    pub view : View,
    pub scroll : ScrolledWindow,
    pub overlay : libadwaita::ToastOverlay,
    pub paned : Paned,
    pub ignore_file_save_action : gio::SimpleAction
}

// set_right_margin
// set_top_margin
// set_left_margin
// set_bottom_margin

impl PapersEditor {

    pub fn build() -> Self {
        let view = View::new();
        view.set_hexpand(true);
        configure_view(&view);
        view.set_width_request(800);
        view.set_halign(Align::Center);
        view.set_hexpand(true);
        view.set_margin_top(98);
        view.set_margin_bottom(98);

        let scroll = ScrolledWindow::new();
        let provider = CssProvider::new();
        provider.load_from_data("* { background-color : #ffffff; } ".as_bytes());

        // scroll.set_kinetic_scrolling(false);

        scroll.style_context().add_provider(&provider, 800);
        scroll.set_child(Some(&view));

        let overlay = libadwaita::ToastOverlay::builder().opacity(1.0).visible(true).build();
        overlay.set_child(Some(&scroll));
        let paned = Paned::new(Orientation::Horizontal);
        let ignore_file_save_action = gio::SimpleAction::new("ignore_file_save", None);

        Self { scroll, view, overlay, paned, ignore_file_save_action }
    }
}

impl React<FileManager> for PapersEditor {

    fn react(&self, manager : &FileManager) {
        manager.connect_opened({
            let view = self.view.clone();
            move |(path, content)| {
                view.buffer().set_text(&content);
            }
        });
        manager.connect_buffer_read_request({
            let view = self.view.clone();
            move |_| -> String {
                let buffer = view.buffer();
                buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    true
                ).to_string()
            }
        });
        manager.connect_close_confirm({
            let overlay = self.overlay.clone();
            move |file| {
                let toast = libadwaita::Toast::builder()
                    .title(&format!("{} has unsaved changes", file))
                    .button_label("Close anyway")
                    .action_name("win.ignore_file_save")
                    .priority(libadwaita::ToastPriority::High)
                    .timeout(0)
                    .build();
                overlay.add_toast(&toast);
            }
        });
    }

}

impl React<Typesetter> for PapersEditor {

    fn react(&self, typesetter : &Typesetter) {
        typesetter.connect_error({
            let overlay = self.overlay.clone();
            move |e| {
                let toast = libadwaita::Toast::builder()
                    .title(&e)
                    .priority(libadwaita::ToastPriority::High)
                    .timeout(0)
                    .build();
                overlay.add_toast(&toast);
            }
        });
    }

}

fn wrap_or_insert_at_cursor(btn : &Button, view : View, popover : Popover, tag : &'static str) {
    btn.connect_clicked(move |_| {
        let buffer = view.buffer();
        if let Some((mut start, mut end)) = buffer.selection_bounds() {
            let prev = buffer.text(&start, &end, true).to_string();
            buffer.delete(&mut start, &mut end);
            buffer.insert(&mut start, &format!("\\{}{{{}}}", tag, prev));
        } else {
            buffer.insert_at_cursor(&format!("\\{}{{}}", tag));
        }
        popover.popdown();
        view.grab_focus();
    });
}

impl React<Titlebar> for PapersEditor {

    fn react(&self, titlebar : &Titlebar) {
        let hide_action = titlebar.sidebar_hide_action.clone();
        let paned = self.paned.clone();
        titlebar.sidebar_toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                let sz = hide_action.state().unwrap().get::<i32>().unwrap();
                if sz > 0 {
                    paned.set_position(sz);
                } else {
                    paned.set_position(320);
                }
            } else {
                hide_action.set_state(&paned.position().to_variant());
                paned.set_position(0);
            }
        });

        let view = &self.view;
        let popover = &titlebar.fmt_popover.popover;
        wrap_or_insert_at_cursor(&titlebar.fmt_popover.bold_btn, view.clone(), popover.clone(), "textbf");
        wrap_or_insert_at_cursor(&titlebar.fmt_popover.underline_btn, view.clone(), popover.clone(), "underline");
        wrap_or_insert_at_cursor(&titlebar.fmt_popover.italic_btn, view.clone(), popover.clone(), "textit");
        wrap_or_insert_at_cursor(&titlebar.fmt_popover.strike_btn, view.clone(), popover.clone(), "sout");
    }
}

impl React<Analyzer> for PapersEditor {

    fn react(&self, analyzer : &Analyzer) {
        let view = self.view.clone();
        analyzer.connect_line_selection(move |line| {
            let buffer = view.buffer();
            if let Some(mut iter) = buffer.iter_at_line(line as i32) {
                buffer.place_cursor(&iter);
                view.scroll_to_iter(&mut iter, 0.0, true, 0.0, 0.5);
                view.grab_focus();
                println!("Cursor placed");
            } else {
                println!("No iter at line {}", line);
            }

            // view.buffer().place_cursor(&iter);
            // view.buffer().move_mark(&mark, &iter);
        });
    }
}

fn configure_view(view : &View) {
    let buffer = view.buffer()
        .downcast::<sourceview5::Buffer>().unwrap();
    let manager = sourceview5::StyleSchemeManager::new();
    let scheme = manager.scheme("Adwaita").unwrap();
    buffer.set_style_scheme(Some(&scheme));
    buffer.set_highlight_syntax(true);
    let provider = CssProvider::new();
    provider.load_from_data(b"textview { font-family: \"Source Code Pro\"; font-size: 13pt; }");
    let ctx = view.style_context();
    ctx.add_provider(&provider, 800);
    let lang_manager = sourceview5::LanguageManager::default().unwrap();
    let lang = lang_manager.language("latex").unwrap();
    buffer.set_language(Some(&lang));
    view.set_tab_width(4);
    view.set_indent_width(4);
    view.set_auto_indent(true);
    view.set_insert_spaces_instead_of_tabs(true);
    view.set_highlight_current_line(false);
    view.set_indent_on_tab(true);
    view.set_show_line_marks(true);
    view.set_enable_snippets(true);
    view.set_wrap_mode(WrapMode::Word);

    // Seems to be working, but only when you click on the the word
    // and **then** press CTRL+Space (simply pressing CTRL+space does not work).
    let completion = view.completion().unwrap();
    let words = sourceview5::CompletionWords::new(Some("main"));
    words.register(&view.buffer());
    completion.add_provider(&words);
    view.set_show_line_numbers(true);
}


