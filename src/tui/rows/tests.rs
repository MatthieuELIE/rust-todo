use super::super::tests::{TODAY, app_of, press, shown};
use super::super::*;

#[test]
fn a_reload_drops_the_fold_of_a_group_gone_from_the_file() {
    let mut app = grouped();
    press(&mut app, "zM");

    app.reload(vec![Todo::from_line("(A) a"), Todo::from_line("d")]);
    app.reload(grouped().todos);

    assert_eq!(
        app.rows(),
        [Row::Group(Group::Priority('A')), Row::Task(3), Row::Group(Group::Unprioritised)]
    );
}

#[test]
fn the_groups_follow_the_priorities_on_screen_with_their_counts() {
    let mut app = app_of(&["(C) one", "two", "(A) three", "x 2026-09-20 four", "(A) five +work"]);
    use Group::*;

    assert_eq!(app.groups(), [(Priority('A'), 2), (Priority('C'), 1), (Unprioritised, 1)]);

    app.filter = Some("+work".to_string());
    assert_eq!(app.groups(), [(Priority('A'), 1)]);
}

#[test]
fn there_are_no_groups_when_no_task_on_screen_has_a_priority() {
    let mut app = app_of(&["one", "x 2026-09-20 two", "(A) three +work"]);

    press(&mut app, "H/-work");
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert_eq!(app.groups(), []);
}

fn grouped() -> App {
    app_of(&["(A) a", "(A) b", "(B) c", "d"])
}

#[test]
fn capital_z_m_folds_every_group_into_a_row_j_and_k_step_through() {
    let mut app = grouped();

    press(&mut app, "jzM");
    assert_eq!(
        app.rows(),
        [
            Row::Group(Group::Priority('A')),
            Row::Group(Group::Priority('B')),
            Row::Group(Group::Unprioritised)
        ]
    );
    assert_eq!(app.cursor, 0);

    press(&mut app, "jjj");
    assert_eq!(app.cursor, 2);
    press(&mut app, "k");
    assert_eq!(app.cursor, 1);
}

#[test]
fn a_folded_header_ignores_the_task_keys() {
    let mut app = grouped();

    press(&mut app, "zM");
    assert!(!press(&mut app, "xddpb"));
    app.handle_key(KeyEvent::from(KeyCode::Enter), TODAY);

    assert!(!matches!(app.focus, Focus::Popup(_)));
    assert_eq!(shown(&app), ["(A) a", "(A) b", "(B) c", "d"]);
}

#[test]
fn capital_z_r_unfolds_back_onto_the_task_or_the_first_task_of_the_folded_group() {
    let mut app = grouped();

    press(&mut app, "jjzM");
    assert_eq!(app.cursor, 1);
    press(&mut app, "zR");
    assert_eq!(app.rows()[app.cursor], Row::Task(3));

    press(&mut app, "zMkzR");
    assert_eq!(app.rows()[app.cursor], Row::Task(1));
    assert_eq!(app.rows().len(), 4);
}

#[test]
fn z_followed_by_another_key_is_dropped_and_zm_without_groups_does_nothing() {
    let mut app = grouped();
    press(&mut app, "zj");
    assert_eq!((app.cursor, app.rows().len()), (1, 4));

    let mut app = app_of(&["one", "two"]);
    press(&mut app, "jzM");
    assert_eq!(app.rows(), [Row::Task(1), Row::Task(2)]);
    assert_eq!(app.cursor, 1);
}

#[test]
fn a_filter_keeps_the_folds_but_a_group_that_leaves_the_screen_comes_back_unfolded() {
    let mut app = grouped();
    let enter = KeyEvent::from(KeyCode::Enter);

    press(&mut app, "zM/a");
    assert_eq!(app.rows(), [Row::Group(Group::Priority('A'))]);
    app.handle_key(enter, TODAY);
    app.handle_key(KeyEvent::from(KeyCode::Esc), TODAY);

    assert_eq!(app.rows(), [Row::Group(Group::Priority('A')), Row::Task(3), Row::Task(4)]);
}

#[test]
fn za_folds_the_group_of_the_task_onto_its_header_and_unfolds_it_onto_its_first_task() {
    let mut app = grouped();

    press(&mut app, "jza");
    assert_eq!(app.rows(), [Row::Group(Group::Priority('A')), Row::Task(3), Row::Task(4)]);
    assert_eq!(app.cursor, 0);

    press(&mut app, "jzMkza");
    assert_eq!(
        app.rows(),
        [
            Row::Task(1),
            Row::Task(2),
            Row::Group(Group::Priority('B')),
            Row::Group(Group::Unprioritised)
        ]
    );
    assert_eq!(app.cursor, 0);
}

#[test]
fn za_without_groups_does_nothing() {
    let mut app = app_of(&["one", "two"]);

    press(&mut app, "jza");

    assert_eq!(app.rows(), [Row::Task(1), Row::Task(2)]);
    assert_eq!(app.cursor, 1);
}

#[test]
fn a_paste_in_the_search_drops_the_folds_of_the_groups_it_takes_off_the_screen() {
    let mut app = app_of(&["(A) one", "two", "three"]);
    press(&mut app, "jza/");

    app.paste("t");

    assert_eq!(app.rows(), [Row::Task(2), Row::Task(3)]);
}
