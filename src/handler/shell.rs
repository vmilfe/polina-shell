use std::collections::HashMap;
use std::env;

use iced::widget::text_editor::Content;

#[derive(Debug)]
pub enum Commands {
    Ls(Vec<String>, Option<HashMap<String, String>>),
    Cd(Vec<String>, Option<HashMap<String, String>>),
    Exit,
    Clear,
    NotFound(String),
}

pub enum SystemCall {
    Display(String),
    Clear,
    Exit,
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
                vec![SystemCall::Display(format!(
                    "Ls directory into {:?}",
                    replaced_args
                ))]
            }
            Commands::Cd(command, extra) => {
                let replaced_args: Vec<String> =
                    self.format_command_args_to_env(&command, extra.as_ref());
                vec![SystemCall::Display(format!(
                    "Changing directory to {:?}",
                    replaced_args
                ))]
            }
            Commands::Exit => {
                vec![SystemCall::Exit]
            }
            Commands::NotFound(command) => {
                vec![SystemCall::Display(format!(
                    "{}: command not found",
                    command
                ))]
            }
            Commands::Clear => {
                vec![SystemCall::Clear]
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
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        match parts[0] {
            "ls" => Commands::Ls(args, None),
            "cd" => Commands::Cd(args, None),
            "exit" => Commands::Exit,
            _ => Commands::NotFound(parts[0].to_string()),
        }
    }
}
