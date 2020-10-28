use crate::config::Config;
use crate::cursive::traits::View;
use crate::model::RepoBranchDeltas;
use crate::utils::execute_on_repo;
use crate::views::{DeltaView, ReposView, SeperatorView};
use cursive::event::{Event, Key};
use cursive::traits::Boxable;
use cursive::traits::Identifiable;
use cursive::views::{BoxView, ViewRef};
use cursive::views::{LayerPosition, LinearLayout};
use cursive::Cursive;
use std::default::Default;

fn update(siv: &mut Cursive, index: usize, repo_deltas: &RepoBranchDeltas) {
    let mut delta_view: ViewRef<DeltaView> = siv.find_id("deltaView").unwrap();
    delta_view.set_repo_deltas(repo_deltas);

    let mut repos_view: ViewRef<ReposView> = siv.find_id("mainView").unwrap();
    repos_view.update_status_bar(index as i32);
}

pub fn show(model: Vec<RepoBranchDeltas>, config: &Config, total_nr_of_repos: usize) {
    let nr_of_filtered_repos = model.len();
    let first_repo = if nr_of_filtered_repos > 0 {
        Some(model.get(0).unwrap().clone())
    } else {
        None
    };

    let mut siv = Cursive::default();
    let screen_size = siv.screen_size();

    let mut repos_view = ReposView::from(model, total_nr_of_repos);

    siv.load_toml(include_str!("../assets/style.toml")).unwrap();

    repos_view.update_status_bar(-1);
    repos_view.set_on_select(
        move |siv: &mut Cursive, _row: usize, index: usize, status: &RepoBranchDeltas| {
            let mut status_view: ViewRef<DeltaView> = siv.find_id("deltaView").unwrap();
            status_view.set_repo_deltas(&status);
            let mut repos_view: ViewRef<ReposView> = siv.find_id("mainView").unwrap();
            repos_view.update_status_bar(index as i32);
        },
    );
    let landscape_format = screen_size.x / (screen_size.y * 3) >= 1;
    let layout = if landscape_format {
        LinearLayout::vertical().child(
            LinearLayout::horizontal()
                .child(repos_view.with_id("mainView").full_screen())
                .child(SeperatorView::vertical())
                .child(BoxView::with_fixed_width(
                    screen_size.x / 2 - 1,
                    DeltaView::empty().with_id("deltaView"),
                )),
        )
    } else {
        LinearLayout::vertical()
            .child(repos_view.with_id("mainView").full_screen())
            .child(BoxView::with_fixed_height(
                screen_size.y / 2 - 1,
                DeltaView::empty().with_id("deltaView"),
            ))
    };

    siv.add_layer(layout);

    register_custom_commands(config, &mut siv);

    register_builtin_command('q', &mut siv, |s| {
        s.pop_layer();
        if s.screen().get(LayerPosition::FromBack(0)).is_none() {
            s.quit();
        }
    });
    register_builtin_command('k', &mut siv, |s| {
        let mut status_view: ViewRef<DeltaView> = s.find_id("deltaView").unwrap();
        status_view.on_event(Event::Key(Key::Up));
    });
    register_builtin_command('j', &mut siv, |s| {
        let mut status_view: ViewRef<DeltaView> = s.find_id("deltaView").unwrap();
        status_view.on_event(Event::Key(Key::Down));
    });

    if let Some(repo) = first_repo {
        update(&mut siv, 0, &repo)
    }
    siv.run();
}

fn register_builtin_command<F>(ch: char, siv: &mut Cursive, cb: F)
where
    F: FnMut(&mut Cursive) + 'static,
{
    siv.clear_global_callbacks(ch); //to avoid that custom commands are taking over one of our builtin shortcuts
    siv.add_global_callback(ch, cb);
}

fn register_custom_commands(config: &Config, siv: &mut Cursive) {
    for cmd in &config.custom_command {
        let executable = cmd.executable.clone();
        let args = cmd.args.clone();

        siv.add_global_callback(cmd.key, move |s| {
            let delta_view: ViewRef<DeltaView> = s.find_id("deltaView").unwrap();
            if let Some(status) = &delta_view.repo_deltas() {
                let result =
                    execute_on_repo(&executable, args.as_ref().unwrap_or(&String::new()), status);
                if let Some(error) = &result.err() {
                    let mut repos_view: ViewRef<ReposView> = s.find_id("mainView").unwrap();
                    repos_view.show_error("Failed to open gitk", error);
                }
            }
        });
    }
}
