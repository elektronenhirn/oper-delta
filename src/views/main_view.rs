use crate::model::RepoDeltas;
use crate::styles::{RED};
use crate::views::table_view::{TableView, TableViewItem};
use cursive::theme::{BaseColor, Color, ColorStyle};
use cursive::traits::*;
use cursive::view::ViewWrapper;
use cursive::views::{Canvas, LinearLayout, ViewRef};
use cursive::Cursive;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Column {
    Repo,
}

impl TableViewItem<Column> for RepoDeltas {
    fn to_column(&self, column: Column) -> String {
        match column {
            Column::Repo => self.repo.rel_path.clone(),
        }
    }

    fn cmp(&self, _other: &Self, _column: Column) -> Ordering
    where
        Self: Sized,
    {
        Ordering::Equal
    }
}

pub struct MainView {
    layout: LinearLayout,
    status_bar_model: Rc<RefCell<String>>,
    number_of_repos: usize,
}

impl MainView {
    pub fn from(model: Vec<RepoDeltas>) -> Self {
        let number_of_repos = model.len();
        let table = Self::new_table(model);
        let status_bar_model = Rc::new(RefCell::new(String::from("")));
        let status_bar = Self::new_status_bar(status_bar_model.clone());

        MainView {
            layout: LinearLayout::vertical()
                .child(table.with_id("table").full_screen())
                .child(status_bar),
            status_bar_model,
            number_of_repos,
        }
    }

    pub fn set_on_select<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive, usize, usize, &RepoDeltas) + 'static,
    {
        let mut table: ViewRef<TableView<RepoDeltas, Column>> =
            self.layout.find_id("table").unwrap();
        table.set_on_select(move |siv: &mut Cursive, row: usize, index: usize| {
            let entry = siv
                .call_on_id("table", move |table: &mut TableView<RepoDeltas, Column>| {
                    table.borrow_item(index).unwrap().clone()
                })
                .unwrap();
            cb(siv, row, index, &entry)
        });
    }

    fn new_table(model: Vec<RepoDeltas>) -> TableView<RepoDeltas, Column> {
        let mut table =
            TableView::<RepoDeltas, Column>::new().column(Column::Repo, "Repo", |c| c.color(*RED));
        table.set_items(model);
        table.set_selected_row(0);

        table
    }

    fn new_status_bar(model: Rc<RefCell<String>>) -> impl cursive::view::View {
        Canvas::new(model)
            .with_draw(|model, printer| {
                let style =
                    ColorStyle::new(Color::Dark(BaseColor::White), Color::Dark(BaseColor::Blue));
                printer.with_style(style, |p| {
                    let text = (*(*model).borrow()).clone();
                    p.print((0, 0), &text);
                    if p.size.x > text.len() {
                        p.print_hline((text.len(), 0), p.size.x - text.len(), " ");
                    }
                });
            })
            .with_required_size(|_model, req| cursive::Vec2::new(req.x, 1))
    }

    pub fn update_status_bar(self: &mut Self, index: usize, _size: usize, _entry: &RepoDeltas) {
        (*self.status_bar_model).replace(format!("Repo {} of {}", index + 1, self.number_of_repos));
    }

    pub fn show_error(self: &mut Self, context: &str, error: &std::io::Error) {
        (*self.status_bar_model).replace(format!("{}: {}", context, error));
    }
}

impl ViewWrapper for MainView {
    type V = LinearLayout;

    fn with_view<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&self.layout))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut self.layout))
    }
}
