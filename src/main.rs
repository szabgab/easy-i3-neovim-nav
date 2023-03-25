use clap::{Parser, Subcommand};
use i3ipc::{reply::Node, I3Connection};
use regex::Regex;

mod focus;
mod resize;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Regex used to extract neovim's v:servername from the window's title string.
    /// The first capture group is used to extract v:servername
    /// The default regex assumes that the servername is contained in square brackets
    /// at the very end of the title string
    #[arg(short, long, default_value_t = Regex::new(r".*\[(.*)\]$").unwrap())]
    extract_server_name: Regex,
}

#[derive(Subcommand)]
enum Commands {
    Focus {
        direction: focus::Direction,
    },
    Resize {
        action: resize::Action,
        dimension: resize::Dimension,
        /// Will be passed as a count to vim's wincmd, see :h wincmd
        #[arg(default_value_t = 5)]
        amount_vim: u32,
        /// Number of pixels by which to resize a floating window,
        /// for details see https://i3wm.org/docs/userguide.html#resizingconfig
        #[arg(default_value_t = 10)]
        amount_i3_px: u32,
        /// Percentage points by which to resize a tiled window,
        /// for details see https://i3wm.org/docs/userguide.html#resizingconfig
        amount_i3_ppt: Option<u32>,
    },
}

fn find_focused_node(node: &Node) -> Option<&Node> {
    if node.focused {
        return Some(node);
    }
    let focus_id = node.focus.first()?;

    node.nodes
        .iter()
        .chain(&node.floating_nodes)
        .find(|n| n.id == *focus_id)
        .and_then(find_focused_node)
}

fn get_nvim_session(tree: &Node, args: &Args) -> Option<neovim_lib::session::Session> {
    let focused = find_focused_node(tree)?;
    let name = focused.name.as_ref()?;

    let path = args.extract_server_name.captures(name)?.get(1)?;

    let mut session = neovim_lib::session::Session::new_unix_socket(path.as_str()).ok()?;
    session.start_event_loop();
    Some(session)
}

fn main() {
    let args = Args::parse();

    let mut i3_connection = I3Connection::connect().unwrap();
    let mut nvim_session = get_nvim_session(&i3_connection.get_tree().unwrap(), &args);

    match args.command {
        Commands::Focus { direction } => {
            focus::switch_focus(direction, nvim_session.as_mut(), &mut i3_connection);
        }
        Commands::Resize {
            action,
            dimension,
            amount_vim,
            amount_i3_px,
            amount_i3_ppt,
        } => {
            resize::resize(
                action,
                dimension,
                &resize::Amounts {
                    vim: amount_vim,
                    i3_px: amount_i3_px,
                    i3_ppt: amount_i3_ppt,
                },
                nvim_session.as_mut(),
                &mut i3_connection,
            );
        }
    };
}
