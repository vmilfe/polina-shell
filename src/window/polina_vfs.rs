use clap::Parser;
use iced::widget::{button, column, text, text_editor, Container};
use iced::{alignment, Element, Length};
use once_cell::sync::Lazy;

use std::process;

use iced::widget::container;
use iced::widget::text_editor::Action;
use iced::widget::text_editor::Edit;

use crate::handler::shell::{Commands, SystemCall};
use crate::vfs::storage::{VFSArgs, VFSNode, VFS};

const SHELL_USER: &str = "ilya";
const OS_NAME: &str = "polina";

pub struct MainWindow {
    text_data: text_editor::Content,
    args: VFSArgs,
    user: String,
    vfs: Option<VFS>,
    history: Vec<String>,
    show_start_button: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    RunStartupScript,
}

impl MainWindow {
    pub fn new() -> MainWindow {
        let shell_args = VFSArgs::parse();
        let vfs = shell_args
            .storage
            .clone()
            .and_then(|path| VFS::new(SHELL_USER.to_string(), path).ok());

        let vfs_user: String;

        match vfs {
            Some(ref vfs) => vfs_user = vfs.user.clone(),
            None => vfs_user = "".to_string(),
        }

        Self {
            text_data: text_editor::Content::with_text(
                Lazy::new(|| MainWindow::get_shell_prefix(vfs_user.clone())).as_str(),
            ),
            args: shell_args.clone(),
            vfs,
            user: vfs_user,
            history: vec![],
            show_start_button: shell_args.startapp.clone().is_some(),
        }
    }

    fn get_shell_prefix(vfs_user: String) -> String {
        format!("[{}@{}]# ", vfs_user, OS_NAME)
    }

    fn custom_message(&mut self, message: &String, start: Option<&String>, end: Option<&String>) {
        let full_message = format!(
            "{}{}{}",
            start.unwrap_or(&"".to_string()),
            message,
            end.unwrap_or(&"".to_string())
        );

        for ch in full_message.chars() {
            self.text_data
                .perform(text_editor::Action::Edit(text_editor::Edit::Insert(ch)));
        }
    }

    pub fn update(&mut self, update: Message) {
        match update {
            Message::Edit(ref message_action) => {
                match message_action {
                    Action::Edit(data_type) => match data_type {
                        Edit::Backspace => {
                            // TODO: very bad method (use Lazy::new), change it in future
                            let text = self
                                .text_data
                                .text()
                                .split(
                                    Lazy::new(|| MainWindow::get_shell_prefix(self.user.clone()))
                                        .as_str(),
                                )
                                .last()
                                .unwrap_or(
                                    Lazy::new(|| MainWindow::get_shell_prefix(self.user.clone()))
                                        .as_str(),
                                )
                                .to_string();

                            if text.len() > 1 {
                                self.text_data.perform(message_action.clone());
                            }
                        }
                        Edit::Enter => {
                            // its bad method, user can write {TERM_PREFIX} and function return bad result
                            let command = Commands::get_last_command_frame(
                                Lazy::new(|| MainWindow::get_shell_prefix(self.user.clone()))
                                    .as_str(),
                                &self.text_data,
                            );
                            self.history.push(command.clone());

                            if self.history.len() > 30 {
                                self.history.remove(0);
                            }

                            let command_result = Commands::parse_from_string(command).execute();

                            for system_call in command_result {
                                match system_call {
                                    SystemCall::ChangeDir(command_args) => {
                                        if self.vfs.is_none() {
                                            self.custom_message(
                                                &format!("VFS storage not set"),
                                                None,
                                                Some(&"\n".to_string()),
                                            );
                                            return;
                                        }
                                        match self.vfs.as_mut() {
                                            Some(vfs) => match vfs.change_dir(command_args) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    self.custom_message(
                                                        &format!("cd: {}", err.to_string()),
                                                        None,
                                                        Some(&"\n".to_string()),
                                                    );
                                                }
                                            },
                                            None => {}
                                        }
                                    }
                                    SystemCall::ListDir(command_args) => {
                                        let dirs_result = match self.vfs.as_mut() {
                                            Some(vfs) => vfs.list_dir(command_args),
                                            None => {
                                                self.custom_message(
                                                    &format!("VFS storage not set"),
                                                    None,
                                                    Some(&"\n".to_string()),
                                                );
                                                return;
                                            }
                                        };

                                        let mut names_vec: Vec<String> = vec![];

                                        match dirs_result {
                                            Ok(dirs) => {
                                                for dir in dirs {
                                                    match dir {
                                                        VFSNode::File { name }
                                                        | VFSNode::Dir { name, .. } => {
                                                            names_vec.push(name.clone());
                                                        }
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                return self.custom_message(
                                                    &format!("ls: {}", err.to_string()),
                                                    None,
                                                    Some(&"\n".to_string()),
                                                );
                                            }
                                        }

                                        if !names_vec.is_empty() {
                                            for name in names_vec {
                                                self.custom_message(
                                                    &name.to_string(),
                                                    None,
                                                    Some(&" ".to_string()),
                                                );
                                            }
                                            self.custom_message(&"\n".to_string(), None, None);
                                        }
                                    }
                                    SystemCall::Whoami => {
                                        self.custom_message(&self.user.clone(), None, None);
                                    }
                                    SystemCall::History => {
                                        for (index, command) in
                                            self.history.clone().iter().enumerate()
                                        {
                                            let mut command = command.clone(); 

                                            if index + 1 == self.history.len() {
                                                command = command.replace("\n", "");
                                            }

                                            self.custom_message(
                                                &format!("{}: {}", index + 1, command).to_string(),
                                                None,
                                                None,
                                            );
                                        }
                                    }
                                    SystemCall::Display(log) => {
                                        self.custom_message(&log, None, None);
                                    }
                                    SystemCall::Exit => {
                                        process::exit(0); // exit code
                                    }
                                    SystemCall::Clear => {
                                        self.text_data = text_editor::Content::new();
                                    }
                                    SystemCall::DisplayNewLine => {
                                        self.custom_message(&"\n".to_string(), None, None);
                                    }
                                    _ => {}
                                }
                            }
                            self.custom_message(
                                &MainWindow::get_shell_prefix(self.user.clone()),
                                None,
                                None,
                            );
                        }
                        _ => {
                            self.text_data.perform(message_action.clone());
                        }
                    },
                    Action::Select(key) => {
                        self.text_data.perform(message_action.clone());
                    }
                    _ => {
                        self.text_data.perform(message_action.clone());
                    }
                }
            }
            Message::RunStartupScript => {
                self.text_data
                    .perform(text_editor::Action::Move(text_editor::Motion::End));

                for command in self.args.get_init_commands() {
                    if command.starts_with('#') {
                        continue;
                    }

                    for char in command.chars() {
                        self.update(Message::Edit(text_editor::Action::Edit(Edit::Insert(char))));
                    }
                    self.update(Message::Edit(text_editor::Action::Edit(Edit::Enter)));
                }

                self.show_start_button = false;
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let title = text("Polina VFS").size(20);
        let subtitle = text("Интерфейс для взаимодействия с виртуальной командной оболочкой")
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let start_button_container: Container<Message> = if let Some(startapp) = &self.args.startapp
        {
            if self.show_start_button {
                container(
                    button(text(format!("Запустить startapp-скрипт: {}", startapp)))
                        .on_press(Message::RunStartupScript),
                )
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right)
                .padding([0, 15])
            } else {
                container(text(""))
            }
        } else {
            container(text(""))
        };

        let title_container: Container<Message> = container(column![title, subtitle])
            .padding([15, 25])
            .width(500);

        let commands_frame = text_editor(&self.text_data)
            .on_action(Message::Edit)
            .height(600);

        let commands_container = container(column![commands_frame]).padding([0, 15]);

        let developer = text("dev: critical")
            .size(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .align_y(alignment::Vertical::Bottom);

        let developer_container: Container<Message> =
            container(column![developer]).padding([15, 15]);

        let interface = column![
            title_container,
            commands_container,
            start_button_container,
            developer_container
        ];

        interface.into()
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}
