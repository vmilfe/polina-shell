use clap::Parser;
use iced::widget::{button, column, text, text_editor, Container};
use iced::{alignment, Element, Length};

use std::process;

use iced::widget::container;
use iced::widget::text_editor::Action;
use iced::widget::text_editor::Edit;

use crate::handler::shell::{Commands, SystemCall};
use crate::vfs::storage::VFSArgs;

const TERM_PREFIX: &str = "ilya@polina# ";

pub struct MainWindow {
    text_data: text_editor::Content,
    args: VFSArgs,
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

        Self {
            text_data: text_editor::Content::with_text(TERM_PREFIX),
            args: shell_args.clone(),
            show_start_button: shell_args.startapp.clone().is_some(),
        }
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
                            let text = self
                                .text_data
                                .text()
                                .split(TERM_PREFIX)
                                .last()
                                .unwrap_or(TERM_PREFIX)
                                .to_string();
                            if text.len() > 1 {
                                self.text_data.perform(message_action.clone());
                            }
                        }
                        Edit::Enter => {
                            // its bad method, user can write {TERM_PREFIX} and function return bad result
                            let command_result = Commands::parse_from_string(
                                Commands::get_last_command_frame(TERM_PREFIX, &self.text_data),
                            )
                            .execute();

                            for system_call in command_result {
                                match system_call {
                                    SystemCall::Display(log) => {
                                        self.custom_message(&log, None, None);
                                    }
                                    SystemCall::Exit => {
                                        process::exit(0); // exit code
                                    }
                                    SystemCall::DisplayNewLine => {
                                        self.custom_message(&"\n".to_string(), None, None);
                                    }
                                    _ => {}
                                }
                            }
                            self.custom_message(&TERM_PREFIX.to_string(), None, None);
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
