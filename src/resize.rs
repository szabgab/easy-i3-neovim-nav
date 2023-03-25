use i3ipc::I3Connection;

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Action {
    Shrink,
    Grow,
}

impl Action {
    fn as_i3_action(&self) -> &str {
        match &self {
            Action::Shrink => "shrink",
            Action::Grow => "grow",
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Dimension {
    Width,
    Height,
}

impl Dimension {
    fn as_i3_dimension(&self) -> &str {
        match &self {
            Dimension::Width => "width",
            Dimension::Height => "height",
        }
    }
}

pub struct Amounts {
    pub vim: u32,
    pub i3_px: u32,
    pub i3_ppt: Option<u32>,
}

trait Resize {
    fn resize(&mut self, action: Action, dimension: Dimension, amounts: &Amounts) -> bool;
}

impl Resize for neovim_lib::Session {
    fn resize(&mut self, action: Action, dimension: Dimension, amounts: &Amounts) -> bool {
        let get_size = match dimension {
            Dimension::Width => "winwidth(winnr())",
            Dimension::Height => "winheight(winnr())",
        };

        let cmd = match (dimension, action) {
            (Dimension::Width, Action::Shrink) => "<",
            (Dimension::Width, Action::Grow) => ">",
            (Dimension::Height, Action::Shrink) => "-",
            (Dimension::Height, Action::Grow) => "+",
        };

        match self.call(
            "nvim_exec",
            vec![
                format!(
                    "let prev = {get_size} | {}wincmd {cmd} | echo ({get_size} == prev)",
                    amounts.vim
                )
                .into(),
                true.into(),
            ],
        ) {
            Err(_) => false,
            Ok(output) => (output.as_str().unwrap()) == "0",
        }
    }
}

impl Resize for I3Connection {
    fn resize(&mut self, action: Action, dimension: Dimension, amounts: &Amounts) -> bool {
        match self.run_command(
            format!(
                "resize {} {} {} px {}",
                action.as_i3_action(),
                dimension.as_i3_dimension(),
                amounts.i3_px,
                match amounts.i3_ppt {
                    None => "".to_string(),
                    Some(ppt) => format!("or {ppt} ppt"),
                }
            )
            .as_str(),
        ) {
            Err(_) => false,
            Ok(reply) => reply.outcomes.first().unwrap().success,
        }
    }
}

pub fn resize(
    action: Action,
    dimension: Dimension,
    amounts: &Amounts,
    nvim_session: Option<&mut neovim_lib::Session>,
    i3_connection: &mut I3Connection,
) {
    match nvim_session.map(|nvim| nvim.resize(action, dimension, amounts)) {
        Some(true) => (),
        _ => {
            i3_connection.resize(action, dimension, amounts);
        }
    }
}
