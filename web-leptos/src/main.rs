use leptos::mount::mount_to_body;

fn main() {
    // tracing_log::LogTracer::init().unwrap();
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    mount_to_body(web_leptos::App);
}
