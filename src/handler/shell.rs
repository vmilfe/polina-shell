use std::collections::HashMap;
use std::{env, vec};

use iced::widget::text_editor::Content;

#[derive(Debug)]
pub enum Commands {
    Ls(Vec<String>, Option<HashMap<String, String>>),
    Cd(Vec<String>, Option<HashMap<String, String>>),
    Chown(Vec<String>),
    Exit,
    Clear,
    Whoami,
    History,
    NotFound(String),
    Null,
}

pub enum SystemCall {
    Display(String),
    Clear,
    Exit,
    Whoami,
    History,
    ChangeDir(Vec<String>),
    ListDir(Vec<String>),
    ChangeOwner(Vec<String>),
    DisplayNewLine,
}

impl Commands {
    fn format_command_args_to_env(
        &self,
        command: &Vec<String>,
        extra: Option<&HashMap<String, String>>,
    ) -> Vec<String> {
        let mut result_vec = Vec::new();

        for arg in command {
            if arg.starts_with('$') {
                let key = arg.trim_start_matches('$');
                let val = env::var(key.replace('$', "")).unwrap_or(arg.clone());
                result_vec.push(val);
                continue;
            }

            if let Some(extra_vars) = extra {
                if let Some(val) = extra_vars.get(arg) {
                    result_vec.push(val.clone());
                    continue;
                }
            }

            result_vec.push(arg.clone());
        }

        result_vec
    }

    pub fn execute(&self) -> Vec<SystemCall> {
        match self {
            Commands::Ls(command, extra) => {
                let replaced_args: Vec<String> =
                    self.format_command_args_to_env(&command, extra.as_ref());

                vec![
                    SystemCall::DisplayNewLine,
                    SystemCall::ListDir(replaced_args),
                ]
            }
            Commands::Cd(command, extra) => {
                let replaced_args: Vec<String> =
                    self.format_command_args_to_env(&command, extra.as_ref());
                vec![
                    SystemCall::DisplayNewLine,
                    SystemCall::ChangeDir(replaced_args.clone()),
                ]
            }
            Commands::Exit => {
                vec![SystemCall::Exit]
            }
            Commands::NotFound(command) => {
                vec![
                    SystemCall::DisplayNewLine,
                    SystemCall::Display(format!("{}: command not found", command)),
                    SystemCall::DisplayNewLine,
                ]
            }
            Commands::Whoami => {
                vec![
                    SystemCall::DisplayNewLine,
                    SystemCall::Whoami,
                    SystemCall::DisplayNewLine,
                ]
            }
            Commands::History => {
                vec![
                    SystemCall::DisplayNewLine,
                    SystemCall::History,
                    SystemCall::DisplayNewLine,
                ]
            }
            Commands::Clear => {
                vec![SystemCall::Clear]
            }
            Commands::Null => {
                vec![SystemCall::DisplayNewLine]
            }
            Commands::Chown(commands) => {
                if commands.len() != 2 {
                    return vec![
                        SystemCall::DisplayNewLine,
                        SystemCall::Display(format!("chown: need 2 arguments")),
                        SystemCall::DisplayNewLine,
                        SystemCall::Display(format!("example: chown root home.pl")),
                        SystemCall::DisplayNewLine,
                    ];
                }

                vec![
                    SystemCall::ChangeOwner(commands.clone()),
                    SystemCall::DisplayNewLine
                ]
            }
        }
    }

    pub fn get_last_command_frame(prefix: &str, data: &Content) -> String {
        let binding = data.text();
        let result: Vec<&str> = binding.split(prefix).collect();
        return result.last().unwrap().to_string();
    }

    pub fn parse_from_string(input: String) -> Commands {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let mut args: Vec<String> = vec![];

        if parts.len() > 0 {
            args = parts[1..].iter().map(|s| s.to_string()).collect();
        }

        if parts.len() == 0 || parts[0].starts_with('#') {
            Commands::Null
        } else {
            match parts[0] {
                "ls" => Commands::Ls(args, None),
                "cd" => Commands::Cd(args, None),
                "exit" => Commands::Exit,
                "clear" => Commands::Clear,
                "whoami" => Commands::Whoami,
                "history" => Commands::History,
                "chown" => Commands::Chown(args),
                _ => Commands::NotFound(parts.get(0).unwrap_or(&"null").to_string()),
            }
        }
    }
}
