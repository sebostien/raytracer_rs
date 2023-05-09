extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .use_colors_if_tty()
        .log_verbose()
        .process_current_dir()
        .unwrap();
}
