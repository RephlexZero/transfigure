mod components;
mod types;
pub mod utils;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(components::App);

    // Drop the static loading placeholder from index.html now the app is up.
    if let Some(boot) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("boot"))
    {
        boot.remove();
    }
}
