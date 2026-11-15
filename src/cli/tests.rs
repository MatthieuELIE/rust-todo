use super::*;

#[test]
fn commands_answer_to_their_short_and_former_names() {
    let command = |args: &[&str]| Cli::try_parse_from(args).unwrap().command.unwrap();

    assert!(matches!(command(&["todo", "ls"]), Commands::List { .. }));
    assert!(matches!(command(&["todo", "a", "Lorem"]), Commands::Add { .. }));
    assert!(matches!(command(&["todo", "rm", "1"]), Commands::Remove { number: 1 }));
    assert!(matches!(command(&["todo", "done", "1"]), Commands::Do { number: 1 }));
}

#[test]
fn a_task_text_may_start_with_a_dash() {
    let command = |args: &[&str]| Cli::try_parse_from(args).unwrap().command.unwrap();

    assert!(matches!(command(&["todo", "add", "-5 degrees"]), Commands::Add { text } if text == "-5 degrees"));
    assert!(matches!(
        command(&["todo", "edit", "2", "-5 degrees"]),
        Commands::Edit { task: Task::Number(2), text } if text == "-5 degrees"
    ));
}

#[test]
fn edit_takes_a_number_made_only_of_digits_and_a_term_otherwise() {
    let command = |args: &[&str]| Cli::try_parse_from(args).unwrap().command.unwrap();

    assert!(matches!(
        command(&["todo", "edit", "2", "Lorem"]),
        Commands::Edit { task: Task::Number(2), .. }
    ));
    assert!(matches!(
        command(&["todo", "edit", "ticket:x/01", "Lorem"]),
        Commands::Edit { task: Task::Term(term), .. } if term == "ticket:x/01"
    ));
    assert!(matches!(
        command(&["todo", "edit", "+2", "Lorem"]),
        Commands::Edit { task: Task::Term(term), .. } if term == "+2"
    ));
    assert!(Cli::try_parse_from(["todo", "edit", "", "Lorem"]).is_err());
}

#[test]
fn list_flags_come_before_terms_which_may_start_with_a_dash() {
    let terms_of = |args: &[&str]| match Cli::try_parse_from(args).unwrap().command {
        Some(Commands::List { due, terms, .. }) => (due, terms),
        _ => panic!("not a list command"),
    };

    assert_eq!(
        terms_of(&["todo", "list", "--due", "+work", "-@home"]),
        (true, vec!["+work".into(), "-@home".into()])
    );
    assert_eq!(
        terms_of(&["todo", "list", "+work", "--due"]),
        (false, vec!["+work".into(), "--due".into()])
    );
    assert!(Cli::try_parse_from(["todo", "list", "--done", "--due"]).is_err());
}
