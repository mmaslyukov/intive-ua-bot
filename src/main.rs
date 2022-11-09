use guardian::bot::telandler::Teladler;
fn main() {

    pretty_env_logger::formatted_timed_builder()
        .filter(Some("guardian"), log::LevelFilter::Trace)
        .init();
    Teladler::new().exec();
}
