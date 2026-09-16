//! Shared terminal and HTML rendering.

mod html;
mod terminal;

pub use html::{HtmlOptions, render_html};
pub use terminal::{TerminalOptions, render_terminal};
