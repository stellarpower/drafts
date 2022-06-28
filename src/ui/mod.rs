use gtk4::*;
use gtk4::prelude::*;
use stateful::React;
use sourceview5::*;
use sourceview5::prelude::ViewExt;
use sourceview5::prelude::BufferExt;
use sourceview5::prelude::CompletionWordsExt;
use tempfile;
use std::rc::Rc;
use std::cell::RefCell;
use crate::manager::FileManager;
use crate::typesetter::{Typesetter, TypesetterTarget};
use gio::prelude::*;
use glib::{types::Type, value::{Value, ToValue}};
use gdk_pixbuf::Pixbuf;
use std::path::Path;
use archiver::SingleArchiverImpl;
use archiver::{OpenDialog, SaveDialog};

mod doctree;

mod titlebar;

mod editor;

pub use titlebar::*;

pub use doctree::*;

pub use editor::*;

#[derive(Debug, Clone)]
pub struct PapersWindow {
    pub window : ApplicationWindow,
    pub titlebar : Titlebar,
    pub editor : PapersEditor,
    pub doc_tree : DocTree,
    pub stack : Stack,
    pub start_screen : StartScreen,
    pub export_pdf_dialog : SaveDialog
}

// \usepackage[utf8]{ulem}

const ARTICLE_TEMPLATE : &'static str = r#"
\documentclass[a4,11pt]{article}

\usepackage[utf8]{inputenc}

\begin{document}
(Article)
\end{document}"#;

const REPORT_TEMPLATE : &'static str = r#"
\documentclass[a4,11pt]{article}

\usepackage[utf8]{inputenc}
(Article)
\begin{document}

\end{document}"#;

const BOOK_TEMPLATE : &'static str = r#"
\begin{document}
\frontmatter

\maketitle

\chapter{Preface}

\mainmatter
\chapter{First chapter}

\appendix
\chapter{First Appendix}

\backmatter
\chapter{Last note}
"#;

const LETTER_TEMPLATE : &'static str = r#"
\documentclass{letter}
\usepackage{hyperref}
\signature{Joe Bloggs}
\address{21 Bridge Street \\ Smallville \\ Dunwich DU3 4WE}
\begin{document}

\begin{letter}{Director \\ Doe \& Co \\ 35 Anthony Road
\\ Newport \\ Ipswich IP3 5RT}
\opening{Dear Sir or Madam:}

\closing{Yours Faithfully,}

\ps

P.S. You can find the full text of GFDL license at
\url{http://www.gnu.org/copyleft/fdl.html}.

\encl{Copyright permission form}

\end{letter}
\end{document}
"#;

const PRESENTATION_TEMPLATE : &'static str = r#"\documentclass{beamer}
\begin{document}
  \begin{frame}
    \frametitle{This is the first slide}
    %Content goes here
  \end{frame}
  \begin{frame}
    \frametitle{This is the second slide}
    \framesubtitle{A bit more information about this}
    %More content goes here
  \end{frame}
% etc
\end{document}
"#;

fn start_document(view : &View, stack : &Stack, titlebar : &Titlebar, template : &str) {
    view.buffer().set_text(ARTICLE_TEMPLATE);
    stack.set_visible_child_name("editor");
    titlebar.main_menu.actions.save.set_enabled(true);
    titlebar.main_menu.actions.save_as.set_enabled(true);
    titlebar.pdf_btn.set_sensitive(true);
}

impl React<StartScreen> for PapersWindow {

    fn react(&self, start_screen : &StartScreen) {

        start_screen.empty_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                view.buffer().set_text("");
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        start_screen.article_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let titlebar = self.titlebar.clone();
            move |_| {
                start_document(&view, &stack, &titlebar, ARTICLE_TEMPLATE);
            }
        });
        /*start_screen.report_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                view.buffer().set_text(REPORT_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        start_screen.presentation_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                view.buffer().set_text(PRESENTATION_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });*/
    }
}

// Create tectonic workspace
// tectonic -X new myfirstdoc

// Compile document in on-off fashion
// tectonic -X compile myfile.tex

// Compile workspace
// tectonic -X build

// tectonic -X compile geometrical-notes.odt --outfmt html

/* Add at the preamble to support Unicode/HTML
\usepackage{fontspec}
\setmainfont{texgyrepagella}[
  Extension = .otf,
  UprightFont = *-regular,
  BoldFont = *-bold,
  ItalicFont = *-italic,
  BoldItalicFont = *-bolditalic,
]
*/

#[derive(Debug, Clone)]
pub struct StartScreen {
    bx : Box,
    empty_btn : Button,
    article_btn : Button,
    //report_btn : Button,
    //presentation_btn : Button
}

pub struct DocBtn {
    pub btn : Button
}

impl DocBtn {

    pub fn build(image : &str, title : &str, sub : &str) -> Self {
        let btn = Button::new();
        //let img = Picture::for_filename(image);
        let img = Picture::for_resource(Some(&format!("com/github/limads/papers/icons/scalable/actions/{}.svg", image)));
        img.set_can_shrink(false);
        let lbl_bx = Box::new(Orientation::Vertical, 12);
        let lbl = Label::new(Some(title));
        lbl.set_justify(Justification::Left);
        lbl.set_halign(Align::Start);
        let sub_lbl = Label::builder().use_markup(true).label(&format!("<span font_weight='normal'>{}</span>", sub)).build();
        sub_lbl.set_halign(Align::Start);
        sub_lbl.set_justify(Justification::Fill);
        lbl_bx.append(&lbl);
        lbl_bx.append(&sub_lbl);

        let bx_btn = Box::new(Orientation::Horizontal, 12);
        bx_btn.append(&img);
        bx_btn.append(&lbl_bx);

        btn.set_child(Some(&bx_btn));
        btn.style_context().add_class("flat");
        btn.set_vexpand(true);
        btn.set_valign(Align::Center);
        btn.set_width_request(480);
        Self { btn }
    }

}

// \url{http://www.uni.edu/~myname/best-website-ever.html}
/*
\begin{comment}
rather stupid,
but helpful
\end{comment}

\textcolor[rgb]{0,1,0}{This text will appear green-colored}

\setmainfont{Georgia}
\setsansfont{Arial}

*/
const PAPERS_PRELUDE : &'static str = r"
    \usepackage{url}
    \usepackage{comment}
    \usepackage{xcolor}
    \usepackage{fontspec}
    \usepackage{multicols}
    \usepackage{amsmath}
";

const EMPTY_DESCRIPTION : &'static str = r#"
Document without a
predefined class"#;

const MINIMAL_DESCRIPTION : &'static str = r#"
Minimal document. Useful for notes, drafts
or any other kind of composition not
requiring sections or metadata."#;

const ARTICLE_DESCRIPTION : &'static str = r#"
Short document divided into sections and
subsections. Aimed at journal articles."#;

const REPORT_DESCRIPTION : &'static str = r#"
Longer document divided into chapters.
Usually aimed at technical reports,
dissertations and thesis."#;

const BOOK_DESCRIPTION : &'static str = r#"
Long document divided into chapters.
Structured into front matter, main matter
and back matter."#;

const PRESENTATION_DESCRIPTION : &'static str = r#"
A document focused on visual presentation.
Separated into slides."#;

impl StartScreen {

    pub fn build() -> Self {
        let doc_upper_bx = Box::new(Orientation::Horizontal, 0);
        let doc_lower_bx = Box::new(Orientation::Horizontal, 0);
        /*let empty_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/empty.svg", "Empty", EMPTY_DESCRIPTION);
        let minimal_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/minimal.svg", "Minimal", MINIMAL_DESCRIPTION);
        let article_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/article.svg", "Article", ARTICLE_DESCRIPTION);
        let report_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/report.svg", "Report", REPORT_DESCRIPTION);
        let book_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/book.svg", "Book", BOOK_DESCRIPTION);
        let present_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/presentation.svg", "Presentation", PRESENTATION_DESCRIPTION);*/
        let empty_btn = DocBtn::build("empty", "Empty", EMPTY_DESCRIPTION);
        let minimal_btn = DocBtn::build("minimal", "Minimal", MINIMAL_DESCRIPTION);
        let article_btn = DocBtn::build("article", "Article", ARTICLE_DESCRIPTION);
        let report_btn = DocBtn::build("report", "Report", REPORT_DESCRIPTION);
        let book_btn = DocBtn::build("book", "Book", BOOK_DESCRIPTION);
        let present_btn = DocBtn::build("presentation", "Presentation", PRESENTATION_DESCRIPTION);

        // let report_btn = Button::builder().label("Report").build();
        // let presentation_btn = Button::builder().label("Presentation").build();
        // letter
        // book

        let center_bx = Box::new(Orientation::Vertical, 32);
        doc_upper_bx.append(&empty_btn.btn);
        doc_upper_bx.append(&minimal_btn.btn);
        doc_upper_bx.append(&article_btn.btn);
        doc_lower_bx.append(&report_btn.btn);
        doc_lower_bx.append(&book_btn.btn);
        doc_lower_bx.append(&present_btn.btn);

        let bx = Box::new(Orientation::Vertical, 0);
        let title = title_label("New document");
        center_bx.append(&title);
        center_bx.append(&doc_upper_bx);
        center_bx.append(&doc_lower_bx);
        //set_margins(&center_bx, 128, 0);
        bx.append(&center_bx);
        center_bx.set_vexpand(true);
        center_bx.set_valign(Align::Center);
        center_bx.set_hexpand(true);
        center_bx.set_halign(Align::Center);

        // bx.append(&report_btn);
        Self { bx, empty_btn : empty_btn.btn.clone(), article_btn : article_btn.btn.clone(), /*report_btn, presentation_btn*/ }
    }

}

const GREEK_SMALL : [(&'static str, &'static str); 24] = [
    ("α", "\\alpha"),
    ("β", "\\beta"),
    ("γ", "\\gamma"),
    ("δ", "\\delta"),
    ("ε", "\\epsilon"),
    ("ζ", "\\zeta"),
    ("η", "\\eta"),
    ("θ", "\\theta"),
    ("ι", "\\iota"),
    ("κ", "\\kappa"),
    ("λ", "\\lambda"),
    ("μ", "\\mu"),
    ("ν", "\\nu"),
    ("ξ", "\\xi"),
    ("ο", "\\omicron"),
    ("π", "\\pi"),
    ("ρ", "\\rho"),
    ("σ", "\\sigma"),
    ("τ", "\\tau"),
    ("υ", "\\upsilon"),
    ("φ", "\\phi"),
    ("χ", "\\chi"),
    ("ψ", "\\psi"),
    ("ω", "\\omega")
];

const GREEK_CAPITAL : [(&'static str, &'static str); 24] = [
    ("Α", "\\Alpha"),
    ("Β", "\\Beta"),
    ("Γ", "\\Gamma"),
    ("Δ", "\\Delta"),
    ("Ε", "\\Epsilon"),
    ("Ζ", "\\Zeta"),
    ("Η", "\\Eta"),
    ("Θ", "\\Theta"),
    ("Ι", "\\Iota"),
    ("Κ", "\\Kappa"),
    ("Λ", "\\Lambda"),
    ("Μ", "\\Mu"),
    ("Ν", "\\Nu"),
    ("Ξ", "\\Xi"),
    ("Ο", "\\Omicron"),
    ("Π", "\\Pi"),
    ("Ρ", "\\Rho"),
    ("Σ", "\\Sigma"),
    ("Τ", "\\Tau"),
    ("Υ", "\\Upsilon"),
    ("Φ", "\\Phi"),
    ("Χ", "\\Chi"),
    ("Ψ", "\\Psi"),
    ("Ω", "\\Omega")
];

// https://en.wikipedia.org/wiki/Mathematical_operators_and_symbols_in_Unicode
const OPERATORS : [(&'static str, &'static str); 48] = [
    ("=", "\\eq"),
    ("⋜", "\\leq"),
    ("⋝", "\\geq"),
    ("≠", "\\neq"),
    ("√", "\\sqrt"),
    (">", ">"),
    ("<", "<"),
    ("×", "\\times"),
    ("÷", "\\div"),
    ("±",  "\\pm"),
    ("∫", "\\int"),
    ("∑", "\\sum"),
    ("⨅", "\\prod"),
    ("→", "\\to"),
    ("↦", "\\mapsto"),
    ("∂", "\\partial"),
    ("∇", "\\nabla"),
    ("∼", "\\tilde"),
    ("∣", "\\vert"),
    ("∘", "\\circ"),
    ("∗", "\\ast"),
    ("∠", "\\angle"),
    ("∀", "\\forall"),
    ("∃", "\\exists"),
    ("∄", "\\nexists"),
    ("∈", "\\in"),
    ("∈/", "\\notin"),
    ("∧", "\\land"),
    ("∨", "\\lor"),
    ("a^", "\\hat"),
    ("△", "\\triangle"),
    ("∴",  "\\therefore"),
    ("∵",  "\\because"),
    ("⋆", "\\star"),
    ("½", "\\frac{}{}"),
    ("∅", "\\emptyset"),
    ("∪", "\\cup"),
    ("∩", "\\cap"),
    ("⋃", "\\bigcup"),
    ("⋂ ", "\\bigcap"),
    ("∖", "\\setminus"),
    ("⊂", "\\sub"),
    ("⊆", "\\sube"),
    ("⊃", "\\supset"),
    ("⊇", "\\supe"),
    ("…",  "\\dots"),
    ("⋱", "\\ddots"),
    ("⋮", "\\vdots"),
];

#[derive(Debug, Clone)]
pub struct SymbolGrid {
    pub grid : Grid
}

impl SymbolGrid {

    pub fn new(symbols : &'static [(&'static str, &'static str)], view : View, popover : Popover, ncol : usize) -> Self {
        let grid = Grid::new();
        set_all_margins(&grid, 6);
        grid.set_margin_bottom(36);
        for row in 0..(symbols.len() / ncol) {
            for col in 0..ncol {
                if let Some(symbol) = symbols.get(row*ncol + col) {
                    let btn = Button::new();
                    btn.set_label(symbol.0);
                    btn.connect_clicked({
                        let view = view.clone();
                        let popover = popover.clone();
                        move|_| {
                            popover.popdown();
                            view.buffer().insert_at_cursor(symbol.1);
                        }
                    });
                    btn.style_context().add_class("flat");
                    grid.attach(&btn, col as i32, row as i32, 1, 1);
                }
            }
        }
        Self { grid }
    }

}

#[derive(Debug, Clone)]
pub struct SymbolPopover {
    pub popover : Popover
}

impl SymbolPopover {

    pub fn build(editor : &PapersEditor) -> Self {
        let popover = Popover::new();
        let greek_small_grid = SymbolGrid::new(&GREEK_SMALL[..], editor.view.clone(), popover.clone(), 12);
        let greek_capital_grid = SymbolGrid::new(&GREEK_CAPITAL[..], editor.view.clone(), popover.clone(), 12);
        let operators_grid = SymbolGrid::new(&OPERATORS[..], editor.view.clone(), popover.clone(), 12);
        let symbol_bx = Box::new(Orientation::Vertical, 0);
        let operators_lbl = Label::builder().label("Operators").halign(Align::Start).build();
        let greek_lbl = Label::builder().label("Greek alphabet").halign(Align::Start).build();
        let greek_capital_lbl = Label::builder().label("Greek alphabet (Capitalized)").halign(Align::Start).build();
        for lbl in [&operators_lbl, &greek_lbl, &greek_capital_lbl] {
            set_all_margins(lbl, 6);
        }
        symbol_bx.append(&operators_lbl);
        symbol_bx.append(&operators_grid.grid);
        symbol_bx.append(&greek_lbl);
        symbol_bx.append(&greek_small_grid.grid);
        symbol_bx.append(&greek_capital_lbl);
        symbol_bx.append(&greek_capital_grid.grid);
        popover.set_child(Some(&symbol_bx));
        Self { popover }
    }
}

// let min_driver = tectonic_bridge_core::MinimalDriver::new(tectonic_io_base::stdstreams::BufferedPrimaryIo::from_buffer(Vec::new()));
// let status = tectonic::status::plain::PlainStatusBackend::new(tectonic::status::ChatterLevel::Minimal);
// tectonic::engines::spx2html::SpxHtmlEngine::new(&mut min_driver, &mut status).process(hooks, status, spx_str);
impl PapersWindow {

    pub fn from(window : ApplicationWindow) -> Self {

        let titlebar = Titlebar::build();
        window.set_titlebar(Some(&titlebar.header));
        window.set_decorated(true);
        let doc_tree = DocTree::build();
        let editor = PapersEditor::build();
        let start_screen = StartScreen::build();

        let export_pdf_dialog = archiver::SaveDialog::build("*.pdf");
        export_pdf_dialog.dialog.set_transient_for(Some(&window));

        titlebar.export_pdf_btn.connect_clicked({
            let export_pdf_dialog = export_pdf_dialog.clone();
            move|btn| {
                export_pdf_dialog.dialog.show();
            }
        });
        export_pdf_dialog.dialog.connect_response({
            let doc = editor.pdf_viewer.doc.clone();
            move |dialog, resp| {
                match resp {
                    ResponseType::Accept => {
                        if let Some(path) = dialog.file().and_then(|f| f.path() ) {
                            if let Some(doc) = &*doc.borrow() {
                                if let Err(e) = doc.save(&format!("file://{}", path.to_str().unwrap())) {
                                    println!("Document save error: {}", e);
                                }
                            } else {
                                println!("No current document to export");
                            }
                        } else {
                            println!("No path available");
                        }
                    },
                    _ => { }
                }
            }
        });

        titlebar.main_menu.save_dialog.dialog.set_transient_for(Some(&window));
        titlebar.main_menu.open_dialog.dialog.set_transient_for(Some(&window));

        // Keeps pdf paned hidden due to window changes. Maybe move to impl React<MainWindow> for Editor?
        window.connect_default_width_notify({
            let paned = editor.sub_paned.clone();
            let pdf_btn = titlebar.pdf_btn.clone();
            move |win| {
                // println!("width changed");
                if !pdf_btn.is_active() || !pdf_btn.is_sensitive() {
                    paned.set_position(i32::MAX);
                }
            }
        });
        window.connect_default_height_notify({
            let paned = editor.sub_paned.clone();
            let pdf_btn = titlebar.pdf_btn.clone();
            move |win| {
                // println!("height changed");
                if !pdf_btn.is_active() || !pdf_btn.is_sensitive() {
                    paned.set_position(i32::MAX);
                }
            }
        });

        window.connect_maximized_notify({
            let paned = editor.sub_paned.clone();
            let pdf_btn = titlebar.pdf_btn.clone();
            move |win| {
                // println!("Maximized changed");
                if !pdf_btn.is_active() || !pdf_btn.is_sensitive() {
                    // let w = win.allocation().width;
                    // println!("{}", w);
                    paned.set_position(i32::MAX);
                }
            }
        });
        window.connect_fullscreened_notify({
            move |win| {
                println!("Fullscreened changed");
            }
        });

        // titlebar.main_menu.open_dialog.react(&titlebar.main_menu);
        titlebar.main_menu.save_dialog.react(&titlebar.main_menu);
        editor.react(&titlebar);
        editor.react(&titlebar.bib_popover);

        // source.set_halign(Align::Center);
        // source.set_margin_start(256);
        // source.set_margin_end(256);

        // let web = webkit2gtk5::WebView::new();
        // web.load_html("<html><head></head><body>Hello world</body></html>", None);
        // web.set_margin_start(18);

        // window.set_child(Some(&editor.overlay));

        // let ws = Rc::new(RefCell::new(Workspace::new()));

        window.add_action(&titlebar.main_menu.actions.new);
        window.add_action(&titlebar.main_menu.actions.open);
        window.add_action(&titlebar.main_menu.actions.save);
        window.add_action(&titlebar.main_menu.actions.save_as);

        window.add_action(&titlebar.sidebar_hide_action);
        window.add_action(&titlebar.zoom_action);
        window.add_action(&editor.ignore_file_save_action);

        let stack = Stack::new();
        stack.add_named(&start_screen.bx, Some("start"));
        stack.add_named(&editor.overlay, Some("editor"));

        editor.paned.set_start_child(&doc_tree.bx);
        editor.paned.set_end_child(&stack);
        editor.paned.set_position(0);

        window.set_child(Some(&editor.paned));

        let symbol_dialog = Dialog::new();
        symbol_dialog.set_title(Some("Symbols"));
        archiver::configure_dialog(&symbol_dialog);
        symbol_dialog.set_transient_for(Some(&window));

        let symbol_popover = SymbolPopover::build(&editor);
        titlebar.symbol_btn.set_popover(Some(&symbol_popover.popover));

        // symbol_dialog.set_child(Some(&symbol_bx));
        // titlebar.math_actions.symbol.connect_activate(move |_, _| {
        //    symbol_dialog.show();
        // });

        let titlebar_actions = titlebar.object_actions.iter()
            .chain(titlebar.layout_actions.iter())
            .chain(titlebar.sectioning_actions.iter())
            .chain(titlebar.block_actions.iter())
            .chain(titlebar.meta_actions.iter())
            .chain(titlebar.indexing_actions.iter());
        for action in titlebar_actions {
            window.add_action(&action);
        }

        Self { window, titlebar, editor, doc_tree, stack, start_screen, export_pdf_dialog }
    }

}

impl React<FileManager> for PapersWindow {

    fn react(&self, manager : &FileManager) {
        archiver::connect_manager_with_app_window_and_actions(manager, &self.window, &self.titlebar.main_menu.actions, "tex");

        // We should trigger a document re-analysis whenever the file is saved.
        // manager.connect_save(move |_| {
        // });

        manager.connect_new({
            let window = self.window.clone();
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            let bib_list = self.titlebar.bib_popover.list.clone();
            let paned = self.editor.sub_paned.clone();
            let titlebar = self.titlebar.clone();
            move |_| {
                window.set_title(Some("Papers"));
                action_save.set_enabled(false);
                action_save_as.set_enabled(false);
                paned.set_position(paned.allocation().width);
                // paned.set_sensitive(false);
                titlebar.set_typeset_mode(false);

                // New files are never linked to references before they are saved.
                titlebar::clear_list(&bib_list);
                titlebar::create_init_row(&bib_list);
            }
        });
        manager.connect_opened({
            let stack = self.stack.clone();
            let paned = self.editor.sub_paned.clone();
            let titlebar = self.titlebar.clone();
            move |(path, _)| {
                stack.set_visible_child_name("editor");
                paned.set_position(paned.allocation().width);
                // paned.set_sensitive(false);
                titlebar.set_typeset_mode(false);
                titlebar.pdf_btn.set_sensitive(true);
            }
        });
        manager.connect_new({
            let stack = self.stack.clone();
            let titlebar = self.titlebar.clone();
            move |_| {
                stack.set_visible_child_name("start");
                titlebar.pdf_btn.set_sensitive(false);
            }
        });
    }

}

impl React<MainMenu> for SaveDialog {

    fn react(&self, menu : &MainMenu) {
        let dialog = self.dialog.clone();
        menu.actions.save_as.connect_activate(move |_,_| {
            dialog.show();
        });
    }

}

#[derive(Debug, Clone)]
pub struct PackedImageLabel  {
    pub bx : Box,
    pub img : Image,
    pub lbl : Label
}

impl PackedImageLabel {

    pub fn build(icon_name : &str, label_name : &str) -> Self {
        let bx = Box::new(Orientation::Horizontal, 0);
        let img = Image::from_icon_name(Some(icon_name));
        let lbl = Label::new(Some(label_name));
        set_margins(&img, 6, 6);
        set_margins(&lbl, 6, 6);
        bx.append(&img);
        bx.append(&lbl);
        Self { bx, img, lbl }
    }

    pub fn extract(bx : &Box) -> Option<Self> {
        let img = get_child_by_index::<Image>(&bx, 0);
        let lbl = get_child_by_index::<Label>(&bx, 1);
        Some(Self { bx : bx.clone(), lbl, img })
    }

    pub fn change_label(&self, label_name : &str) {
        self.lbl.set_text(label_name);
    }

    pub fn change_icon(&self, icon_name : &str) {
        self.img.set_icon_name(Some(icon_name));
    }

}

pub fn set_border_to_title(bx : &Box) {
    let provider = CssProvider::new();
    provider.load_from_data("* { border-bottom : 1px solid #d9dada; } ".as_bytes());
    bx.style_context().add_provider(&provider, 800);
}

pub fn try_get_child_by_index<W>(w : &Box, pos : usize) -> Option<W>
where
    W : IsA<glib::Object>
{
    w.observe_children().item(pos as u32)?.clone().downcast::<W>().ok()
}

pub fn get_child_by_index<W>(w : &Box, pos : usize) -> W
where
    W : IsA<glib::Object>
{
    // w.observe_children().item(pos as u32).unwrap().clone().downcast::<W>().unwrap()
    try_get_child_by_index::<W>(w, pos).unwrap()
}

pub fn set_margins<W : WidgetExt>(w : &W, horizontal : i32, vertical : i32) {
    w.set_margin_start(horizontal);
    w.set_margin_end(horizontal);
    w.set_margin_top(vertical);
    w.set_margin_bottom(vertical);
}

pub fn set_all_margins<W : WidgetExt>(w : &W, margin : i32) {
    w.set_margin_start(margin);
    w.set_margin_end(margin);
    w.set_margin_top(margin);
    w.set_margin_bottom(margin);
}

pub fn setup_position_as_ratio(win : &ApplicationWindow, paned : &Paned, ratio : f32) {
    let ratio = start_position_as_ratio(win, paned, ratio);
    preserve_ratio_on_resize(win, paned, &ratio);
}

fn start_position_as_ratio(win : &ApplicationWindow, paned : &Paned, ratio : f32) -> Rc<RefCell<f32>> {
    let paned = paned.clone();
    win.connect_show(move |win| {
        set_position_as_ratio(win, &paned, ratio);
    });
    Rc::new(RefCell::new(ratio))
}

fn set_position_as_ratio(win : &ApplicationWindow, paned : &Paned, ratio : f32) {
    let (mut w, mut h) = (win.allocation().width, win.allocation().height);

    // Allocation will be zero at the first time window is shown.
    if w == 0 {
        w = win.default_width();
    }
    if h == 0 {
        h = win.default_height();
    }
    let dim = match paned.orientation() {
        Orientation::Horizontal => w as f32,
        Orientation::Vertical => h as f32,
        _ => { return; }
    };
    println!("({:?})", dim);
    paned.set_position((dim * ratio) as i32);
}

fn update_paned_from_allocation(win : &ApplicationWindow, paned : &Paned, ratio : &Rc<RefCell<f32>>) {
    let alloc = win.allocation();
    set_position_as_ratio(&win, &paned, *ratio.borrow());
}

fn preserve_ratio_on_resize(win : &ApplicationWindow, paned : &Paned, ratio : &Rc<RefCell<f32>>) {
    win.connect_default_width_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        move |win| {
            update_paned_from_allocation(&win, &paned, &ratio);
        }
    });
    win.connect_default_height_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        move |win| {
            update_paned_from_allocation(&win, &paned, &ratio);
        }
    });
    win.connect_maximized_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        move |win| {
            update_paned_from_allocation(&win, &paned, &ratio);
        }
    });
    win.connect_resizable_notify({

        move |win| {
            println!("Resizable");
        }
    });
    let ratio = ratio.clone();
    let win = win.clone();
    paned.connect_accept_position(move |paned| {
        let dim = match paned.orientation() {
            Orientation::Horizontal => win.allocation().width as f32,
            Orientation::Vertical => win.allocation().height as f32,
            _ => { return true; }
        };
        let new_ratio = paned.position() as f32 / dim;
        *(ratio.borrow_mut()) = new_ratio;
        true
    });
}

const A4 : (f64, f64) = (210.0, 297.4);

const LETTER : (f64, f64) = (215.9, 279.4);

const LEGAL : (f64, f64) = (215.9, 355.6);

// const PX_PER_MM : f64 = 3.0;

impl React<Typesetter> for PapersWindow {

    fn react(&self, typesetter : &Typesetter) {
        let win = self.window.clone();
        let editor = self.editor.clone();
        let titlebar = self.titlebar.clone();
        typesetter.connect_done(move |target| {
            match target {
                TypesetterTarget::File(path) => {

                    // #[cfg(feature="poppler-rs")]
                    // {
                    show_with_poppler(&editor.pdf_viewer, &titlebar.zoom_action, &win, &path[..]);
                    println!("Showing with poppler");
                    // }

                    // println!("Not showing with poppler");
                    // show_with_evince(&path);

                    editor.sub_paned.set_position(editor.sub_paned.allocation().width / 2);
                    // editor.sub_paned.set_sensitive(true);
                    titlebar.set_typeset_mode(true);

                    // If sidebar is open, use minimum zoom at PDF to minimize occlusion of content.
                    if titlebar.sidebar_toggle.is_active() {
                        while titlebar.zoom_out_btn.is_sensitive() {
                            titlebar.zoom_out_btn.emit_clicked();
                        }
                    }
                },
                _ => {

                }
            }
        });
    }

}

fn show_with_evince(path : &str) {

    use std::process::Command;

    let out = Command::new("evince")
        .args(&[&path])
        .spawn()
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct PdfViewer {
    scroll : ScrolledWindow,
    pages_bx : Box,
    das : Rc<RefCell<Vec<DrawingArea>>>,
    doc : Rc<RefCell<Option<poppler::Document>>>
}

impl React<Titlebar> for PdfViewer {
    fn react(&self, titlebar : &Titlebar) {
        titlebar.zoom_action.connect_activate({
            let das = self.das.clone();
            move |_,_| {
                das.borrow().iter().for_each(|da| da.queue_draw() );
            }
        });
        titlebar.pdf_btn.connect_toggled({
            let viewer = self.clone();
            move |btn| {
                if !btn.is_active() {
                    viewer.clear_pages();
                }
            }
        });
    }
}

// Equivalent to 0xdc
// const PAGE_BORDER_COLOR : f64 = 0.859375;

// Equivalent to 0xcf
pub const PAGE_BORDER_COLOR : f64 = 0.80859375;

pub const PAGE_BORDER_WIDTH : f64 = 0.5;

impl PdfViewer {

    pub fn clear_pages(&self) {
        while let Some(child) = self.pages_bx.last_child() {
            self.pages_bx.remove(&child);
        }
        self.doc.replace(None);
    }

    pub fn new() -> Self {
        let scroll = ScrolledWindow::new();
        let pages_bx = Box::new(Orientation::Vertical, 12);
        scroll.set_child(Some(&pages_bx));
        let das = Rc::new(RefCell::new(Vec::new()));
        Self { scroll, das, pages_bx, doc : Rc::new(RefCell::new(None)) }
    }

    pub fn update(&self, doc : &poppler::Document, zoom_action : &gio::SimpleAction) {

        {
            self.das.borrow_mut().clear();
        }

        self.clear_pages();
        for page_ix in 0..doc.n_pages() {
            let da = DrawingArea::new();
            // let zoom = zoom.clone();
            crate::draw_page_at_area(doc, page_ix, &da, zoom_action);
            self.pages_bx.append(&da);
            self.das.borrow_mut().push(da);
        }
        self.doc.replace(Some(doc.clone()));
    }

}

// #[cfg(feature="poppler")]
fn show_with_poppler(viewer : &PdfViewer, zoom_action : &gio::SimpleAction, win : &ApplicationWindow, path : &str) {
    let doc = poppler::Document::from_file(&format!("file://{}", path), None).unwrap();
    viewer.update(&doc, zoom_action);
    // let dialog = Dialog::new();
    // dialog.set_default_width(1024);
    // dialog.set_default_height(768);
    // dialog.set_transient_for(Some(win));
    // let header = HeaderBar::new();
    // header.pack_start(&viewer.zoom_bx);
    // dialog.set_titlebar(Some(&header));
    // dialog.set_title(Some(&Path::new(&path).file_name().unwrap().to_str().unwrap()));
    // dialog.set_child(Some(&viewer.scroll));
    // set_margins(&bx, 32, 32);
    // dialog.show();
}

pub fn title_label(txt : &str) -> Label {
    let lbl = Label::builder()
        .label(&format!("<span font_weight=\"600\" font_size=\"large\" fgalpha=\"60%\">{}</span>", txt))
        .use_markup(true)
        .justify(Justification::Left)
        .halign(Align::Start)
        .build();
    set_margins(&lbl, 0, 12);
    lbl
}

pub fn connect_toast_dismissed(t : &libadwaita::Toast, last : &Rc<RefCell<Option<libadwaita::Toast>>>) {
    let last = last.clone();
    t.connect_dismissed(move|_| {
        if let Ok(mut last) = last.try_borrow_mut() {
            *last = None;
        }
    });
}

pub const MARGIN_MAX : f64 = 5.0;

pub const MARGIN_MIN : f64 = 0.0;

pub struct PaperMargins {
    pub left : f64,
    pub top : f64,
    pub right : f64,
    pub bottom : f64
}

pub fn parse_int_or_float(txt : &str) -> Option<f64> {
    if let Ok(val) = txt.parse::<f64>() {
        Some(val)
    } else {
        if let Ok(val) = txt.parse::<i64>() {
            Some(val as f64)
        } else {
            None
        }
    }
}

impl React<FileManager> for OpenDialog {

    fn react(&self, manager : &FileManager) {
        let dialog = self.dialog.clone();
        manager.connect_show_open(move |_| {
            dialog.show();
        });
    }

}

impl React<FileManager> for SaveDialog {

    fn react(&self, manager : &FileManager) {
        let dialog = self.dialog.clone();
        manager.connect_save_unknown_path(move |path| {
            // let _ = dialog.set_file(&gio::File::for_path(path));
            dialog.show();
        });
        // let dialog = self.dialog.clone();
        /*scripts.connect_path_changed(move |opt_file| {
            if let Some(path) = opt_file.and_then(|f| f.path.clone() ) {
                let _ = dialog.set_file(&gio::File::for_path(&path));
            }
        });*/
    }

}

