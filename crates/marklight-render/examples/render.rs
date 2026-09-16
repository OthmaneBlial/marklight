fn main() {
    let doc = marklight_core::Document::parse(include_str!("../../../fixtures/markdown/gfm.md"));
    print!(
        "{}",
        marklight_render::render_terminal(
            &doc,
            marklight_render::TerminalOptions {
                ansi: false,
                width: 60,
                ..Default::default()
            }
        )
    );
}
