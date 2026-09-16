use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

use clap::{Parser, Subcommand, ValueEnum};
use marklight_core::{Document, load_file, read_source, resolve_document};
use marklight_render::{TerminalOptions, render_terminal};

#[derive(Parser)]
#[command(
    version,
    about = "Read Markdown comfortably in your terminal or desktop",
    after_help = "EXAMPLES:\n  marklight README.md              Read in the terminal\n  marklight .                      Read this directory's README\n  git show HEAD:README.md | marklight -\n  marklight open README.md         Open the desktop reader\n  marklight README.md --gui        Open the desktop reader\n  marklight README.md --plain --no-pager > readme.txt\n\nLong documents use $PAGER (default: less -R). In less: / searches, n goes\nto the next match, and q quits. Set MARKLIGHT_DESKTOP to a desktop binary\npath when the desktop app is not installed."
)]
struct Args {
    #[command(subcommand)]
    command: Option<Action>,
    /// Markdown file, directory (README), or - for stdin
    file: Option<PathBuf>,
    /// Open in the Marklight desktop reader
    #[arg(long, global = true)]
    gui: bool,
    /// Remove all ANSI styles (also automatic when redirected)
    #[arg(long, global = true)]
    plain: bool,
    /// Print directly instead of using a pager
    #[arg(long, global = true)]
    no_pager: bool,
    /// Terminal output width in columns (20–240)
    #[arg(long, global = true, value_parser = clap::value_parser!(u16).range(20..=240))]
    width: Option<u16>,
    /// Colour palette; system uses the terminal background hint
    #[arg(long, global = true, value_enum, default_value = "system")]
    theme: Theme,
}

#[derive(Subcommand)]
enum Action {
    /// Open a file or directory in the desktop application
    Open { file: PathBuf },
}

#[derive(Clone, Copy, ValueEnum)]
enum Theme {
    System,
    Light,
    Dark,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Marklight: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Args) -> io::Result<()> {
    let gui = args.gui || args.command.is_some();
    let file = match args.command {
        Some(Action::Open { file }) => file,
        None => args.file.unwrap_or_else(|| ".".into()),
    };
    if gui {
        if file == Path::new("-") {
            return Err(io::Error::other(
                "stdin is supported in terminal mode; open a saved file for the desktop",
            ));
        }
        let path = resolve_document(&file).map_err(io::Error::other)?;
        return launch_desktop(&path);
    }
    let document = if file == Path::new("-") {
        let source = read_source(io::stdin().lock()).map_err(io::Error::other)?;
        Document::parse(&source)
    } else {
        load_file(&file).map_err(io::Error::other)?.1
    };
    let terminal = io::stdout().is_terminal();
    let size = terminal_size::terminal_size();
    let width = args.width.map(usize::from).unwrap_or_else(|| {
        size.map(|(w, _)| usize::from(w.0).clamp(20, 240))
            .unwrap_or(80)
    });
    let dark = match args.theme {
        Theme::Light => false,
        Theme::Dark => true,
        Theme::System => std::env::var("COLORFGBG")
            .ok()
            .and_then(|v| v.rsplit(';').next()?.parse::<u8>().ok())
            .is_none_or(|v| v < 7),
    };
    let output = render_terminal(
        &document,
        TerminalOptions {
            width,
            ansi: terminal && !args.plain && std::env::var_os("NO_COLOR").is_none(),
            dark,
        },
    );
    let height = size.map(|(_, h)| usize::from(h.0)).unwrap_or(24);
    if terminal && !args.no_pager && output.lines().count() > height.saturating_sub(2) {
        page(&output)?;
    } else {
        io::stdout().lock().write_all(output.as_bytes())?;
    }
    Ok(())
}

fn page(output: &str) -> io::Result<()> {
    let configured = std::env::var("PAGER").unwrap_or_else(|_| "less -R".into());
    let mut argv = shell_words::split(&configured)
        .map_err(|_| io::Error::other("$PAGER has invalid quoting"))?;
    if argv.is_empty() {
        return io::stdout().lock().write_all(output.as_bytes());
    }
    let executable = argv.remove(0);
    // Pass styles through less even when the user sets PAGER=less. Do not use a shell.
    if Path::new(&executable)
        .file_name()
        .is_some_and(|name| name == "less")
        && !argv.iter().any(|a| a == "-R")
    {
        argv.push("-R".into());
    }
    let mut child = match Command::new(&executable)
        .args(argv)
        .env("LESSCHARSET", "utf-8")
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            eprintln!(
                "Marklight: pager {executable:?} is unavailable ({error}); printing directly"
            );
            return io::stdout().lock().write_all(output.as_bytes());
        }
    };
    let result = child.stdin.take().unwrap().write_all(output.as_bytes());
    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::other(format!("pager exited with {status}")));
    }
    match result {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => Err(e),
        _ => Ok(()),
    }
}

fn launch_desktop(path: &Path) -> io::Result<()> {
    if let Some(binary) = std::env::var_os("MARKLIGHT_DESKTOP") {
        Command::new(binary)
            .arg(path)
            .spawn()
            .map_err(|e| io::Error::other(format!("unable to launch MARKLIGHT_DESKTOP: {e}")))?;
        return Ok(());
    }
    let sibling = std::env::current_exe()?.with_file_name(if cfg!(windows) {
        "marklight-desktop.exe"
    } else {
        "marklight-desktop"
    });
    if sibling.is_file() {
        Command::new(sibling).arg(path).spawn()?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .args(["-a", "Marklight"])
            .arg(path)
            .status()?;
        if status.success() {
            return Ok(());
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        match Command::new("marklight-desktop").arg(path).spawn() {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other(
        "desktop app is not installed; install Marklight or set MARKLIGHT_DESKTOP to its executable",
    ))
}
