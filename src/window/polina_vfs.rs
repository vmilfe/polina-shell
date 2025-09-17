use iced::futures::future::Select;
use iced::widget::{
    center, column, combo_box, scrollable, text, text_editor, vertical_space, Container,
};
use iced::{Application, Center, Element, Fill};

use std::env;
use std::default;
use std::process;

use iced::widget::container::Style;
use iced::widget::text_editor::Action;
use iced::widget::text_editor::Edit;
use iced::widget::{button, container, text_input, Column};
use iced::{Color, Theme};

use crate::handler::shell::{Commands, SystemCall};

pub struct MainWindow {
    text_data: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
}
const TERM_PREFIX: &str = "ilya@polina# ";


impl MainWindow {
    pub fn new() -> MainWindow {
        Self {
            text_data: text_editor::Content::with_text(TERM_PREFIX),
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
                            let text = self.text_data.text();
                            if text.len() - 1 > TERM_PREFIX.len() {
                                self.text_data.perform(message_action.clone());
                            }
                        }
                        /* Edit::Insert(pressed_char) => {
                            println!("{}", pressed_char);
                            let mut text = self.text_data.text();
                            text.push(*pressed_char);
                        } */
                        Edit::Enter => {
                            // its bad method, user can write {TERM_PREFIX} and function return bad result
                            let command_result = Commands::parse_from_string(
                                Commands::get_last_command_frame(TERM_PREFIX, &self.text_data),
                            )
                            .execute();
                            
                            for system_call in command_result {
                                match system_call {
                                    SystemCall::Display(log) => {
                                        self.custom_message(&log, Some(&"\n".to_string()), None);
                                        self.custom_message(
                                            &TERM_PREFIX.to_string(),
                                            Some(&"\n".to_string()),
                                            None,
                                        );
                                    },
                                    SystemCall::Exit => {
                                        process::exit(0); // exit code
                                    }
                                    _ => {}
                                }
                            }
                            
                            
                        }
                        _ => {
                            self.text_data.perform(message_action.clone());
                        }
                    },
                    Action::Select(key) => {
                        self.text_data.perform(message_action.clone());
                        println!("{:?} motion", key);
                    }
                    _ => {
                        println!("{:?} from under call", update);
                        self.text_data.perform(message_action.clone());
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let title = text("Polina VFS").size(20);
        let subtitle = text("Интерфейс для взаимодействия с виртуальной командной оболочкой")
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let title_container: Container<Message> = container(column![title, subtitle])
            .padding([15, 25])
            .width(500);

        let commands_frame = text_editor(&self.text_data)
            .on_action(Message::Edit)
            .height(600);

        let commands_container = container(column![commands_frame]).padding([0, 15]);

        let interface = column![title_container, commands_container];

        interface.into()
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}
