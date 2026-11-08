use super::*;

fn todos_of(lines: &[&str]) -> Vec<Todo> {
    lines.iter().map(|l| Todo::from_line(l)).collect()
}

fn listed(todos: &[Todo], terms: &[&str]) -> Vec<(usize, String)> {
    let terms: Vec<String> = terms.iter().map(|t| t.to_string()).collect();
    list(todos, &terms).into_iter().map(|(n, t)| (n, t.to_line())).collect()
}

#[test]
fn a_filtered_list_keeps_the_numbers_of_the_file() {
    let todos = todos_of(&["x Lorem ipsum", "Consectetur adipiscing", "Tempor incididunt"]);

    assert_eq!(
        listed(&todos, &["-lorem"]),
        [(2, "Consectetur adipiscing".into()), (3, "Tempor incididunt".into())]
    );
    assert_eq!(listed(&todos, &[]).len(), 3);
}

#[test]
fn every_term_must_appear_in_the_line() {
    let todos = todos_of(&["Call the bank +finance", "Pay rent +finance @home", "Call mom @phone"]);

    assert_eq!(listed(&todos, &["+finance", "Call"]), [(1, "Call the bank +finance".into())]);
}

#[test]
fn a_leading_dash_excludes_lines_containing_the_term() {
    let todos = todos_of(&["Call the bank +finance", "Pay rent +finance @home", "Call mom @phone"]);

    assert_eq!(listed(&todos, &["+finance", "-@home"]), [(1, "Call the bank +finance".into())]);
    assert_eq!(listed(&todos, &["-"]).len(), 3);
}

#[test]
fn terms_ignore_case() {
    let todos = todos_of(&["Call the BANK", "Pay rent @Home"]);

    assert_eq!(listed(&todos, &["bank"]), [(1, "Call the BANK".into())]);
    assert_eq!(listed(&todos, &["-@HOME"]), [(1, "Call the BANK".into())]);
}

#[test]
fn list_matches_its_terms_on_the_whole_line_priority_and_dates_included() {
    let todos = todos_of(&["(A) 2026-09-01 Lorem", "Ipsum"]);

    assert_eq!(listed(&todos, &["(a)"]), [(1, "(A) 2026-09-01 Lorem".into())]);
    assert_eq!(listed(&todos, &["2026-09"]), [(1, "(A) 2026-09-01 Lorem".into())]);
}

#[test]
fn list_puts_priorities_first_then_unprioritised_each_in_file_order() {
    let todos = todos_of(&["Lorem", "Ipsum", "(B) Dolor", "Sit", "(A) Amet", "(B) Elit"]);

    let numbers: Vec<usize> = listed(&todos, &[]).into_iter().map(|(n, _)| n).collect();

    assert_eq!(numbers, [5, 3, 6, 1, 2, 4]);
}
