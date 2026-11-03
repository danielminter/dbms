use dbms::evaluate_query;
use dbms::get_input;
use dbms::print_prompt;

fn main() {
    loop {
        print_prompt();

        let user_input = get_input();

        if user_input == ".exit" {
            break;
        } else if user_input == ".help" {
            println!("Simple DBMS REPL.");
            println!("Commands:");
            println!(".exit     Exit the REPL");
            println!(".help     This menu");
        } else {
            let query_status = evaluate_query(&user_input).unwrap();
            println!("Status: {}", query_status);
        }
    }
}
