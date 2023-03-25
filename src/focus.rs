use clap::ValueEnum;
use i3ipc::I3Connection;

#[derive(ValueEnum, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn as_nvim_direction(&self) -> &str {
        match self {
            Direction::Left => "h",
            Direction::Right => "l",
            Direction::Up => "k",
            Direction::Down => "j",
        }
    }

    fn as_i3_direction(&self) -> &str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
}

trait SwitchFocus {
    fn switch_focus(&mut self, dir: Direction) -> bool;
}

impl SwitchFocus for neovim_lib::Session {
    fn switch_focus(&mut self, dir: Direction) -> bool {
        match self.call(
            "nvim_exec",
            vec![
                format!(
                    "let prev = winnr() | wincmd {} | echo (winnr() == prev)",
                    dir.as_nvim_direction()
                )
                .into(),
                true.into(),
            ],
        ) {
            Err(_) => false,
            Ok(output) => output.as_str().unwrap() == "0",
        }
    }
}

impl SwitchFocus for i3ipc::I3Connection {
    fn switch_focus(&mut self, dir: Direction) -> bool {
        match self.run_command(format!("focus {}", dir.as_i3_direction()).as_str()) {
            Err(_) => false,
            Ok(reply) => reply.outcomes.first().unwrap().success,
        }
    }
}

pub fn switch_focus(
    dir: Direction,
    nvim_session: Option<&mut neovim_lib::Session>,
    i3_connection: &mut I3Connection,
) {
    match nvim_session.map(|nvim| nvim.switch_focus(dir)) {
        Some(true) => (),
        _ => {
            i3_connection.switch_focus(dir);
        }
    }
}
