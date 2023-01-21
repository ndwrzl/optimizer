use egui::{Align2, Ui};

use lazy_static::lazy_static;

use eframe::egui;
use std::{
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};
use structs::{Action, Kill, KillParse, Types};

use crate::structs::KillService;
mod kill;
mod structs;

lazy_static! {
    static ref TO_KILL: &'static Path = Path::new("kill.json");
}

const WINDOW_W: f32 = 640.;
const WINDOW_H: f32 = 400.;

const POPAP_W: f32 = 500.;
const POPAP_H: f32 = 350.;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(WINDOW_W, WINDOW_H)),
        always_on_top: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        resizable: false,
        ..Default::default()
    };
    eframe::run_native("Optimizer", options, Box::new(|_cc| Box::new(MyApp::new())))
}

#[derive(Default)]
struct MyApp {
    show_action_confirmation: bool,
    chosen_action: Action,
    show_message: bool,
    result: String,
    data: KillParse,
    adding_data: Kill,
    adding: bool,
    adding_service: Types,
    edit: Kill,
    edit_service: KillService,
    editing_index: Option<usize>,
    editing_service: bool,
}

impl MyApp {
    fn new() -> Self {
        let kill_processes = match read_to_string(*TO_KILL) {
            Ok(x) => x,
            Err(error) => {
                eprintln!("{error}");
                String::new()
            }
        };
        let data = match serde_json::from_str::<KillParse>(&kill_processes) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("{e}");
                KillParse::default()
            }
        };

        MyApp {
            data,
            ..Default::default()
        }
    }

    fn confirmation_dialog(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut style = ui.style_mut();
            style.spacing.button_padding = (16., 8.).into();

            if ui.button("Yes!").clicked() {
                self.result = match self.chosen_action {
                    Action::Kill => match kill::kill(&self.data) {
                        Ok(e) => e,
                        Err(e) => e,
                    },

                    Action::Restore => match kill::restore() {
                        Ok(e) => e,
                        Err(e) => e,
                    },
                };
                self.show_message = true;
                self.show_action_confirmation = false;
            };
            if ui.button("No").clicked() {
                self.show_action_confirmation = false;
            };
        });
    }

    fn edit_window(&mut self, ui: &mut Ui, editing: usize) {
        let mut style = ui.style_mut();
        style.spacing.button_padding = (16., 8.).into();
        egui::Grid::new("my_grid")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                if self.editing_service {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.edit_service.name);
                    ui.end_row();
                    ui.label("Enabled");
                    ui.checkbox(&mut self.edit_service.enabled, "");
                    ui.end_row();
                    ui.label("Restore");
                    ui.checkbox(&mut self.edit_service.restore, "");
                } else {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.edit.name);
                    ui.end_row();
                    ui.label("Enabled");
                    ui.checkbox(&mut self.edit.enabled, "");
                    ui.end_row();
                    ui.label("Restore");
                    ui.checkbox(&mut self.edit.restore, "");
                    ui.end_row();
                    ui.label("Admin");
                    ui.checkbox(&mut self.edit.admin, "");
                }
            });
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                if self.editing_service {
                    self.data.services[editing] = self.edit_service.clone();
                } else {
                    self.data.processes[editing] = self.edit.clone();
                }
                self.editing_index = None;
            }
            if ui.button("Close").clicked() {
                self.editing_index = None;
            }
        });
    }

    fn process_table(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};
        ui.vertical(|ui| {
            egui::ScrollArea::vertical().max_width(430.).show(ui, |ui| {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::initial(55.))
                    .column(Column::remainder())
                    .column(Column::initial(55.))
                    .column(Column::initial(55.))
                    .column(Column::initial(55.));

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Enabled");
                        });
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Restore");
                        });
                        header.col(|ui| {
                            ui.strong("Admin");
                        });
                        header.col(|_| {});
                    })
                    .body(|mut body| {
                        let mut index = 0;
                        self.data.processes.retain_mut(|process| {
                            let mut keep = true;
                            body.row(30., |mut row| {
                                row.col(|ui| {
                                    ui.checkbox(&mut process.enabled, "");
                                });
                                row.col(|ui| {
                                    ui.label(&process.name);
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut process.restore, "");
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut process.admin, "");
                                });
                                row.col(|ui| {
                                    ui.horizontal_centered(|ui| {
                                        if ui.button("✏").clicked() {
                                            self.edit = process.clone();
                                            self.editing_index = Some(index);
                                            self.editing_service = false;
                                        };
                                        if ui.button("🗑").clicked() {
                                            keep = false;
                                        };
                                    });
                                });
                                index += 1;
                            });
                            keep
                        });
                        self.data.services.retain_mut(|service| {
                            let mut keep = true;
                            body.row(30., |mut row| {
                                row.col(|ui| {
                                    ui.checkbox(&mut service.enabled, "");
                                });
                                row.col(|ui| {
                                    ui.label(&service.name);
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut service.restore, "");
                                });
                                row.col(|ui| {
                                    ui.label("Service");
                                });
                                row.col(|ui| {
                                    ui.horizontal_centered(|ui| {
                                        if ui.button("✏").clicked() {
                                            self.edit_service = service.clone();
                                            self.editing_index = Some(index);
                                            self.editing_service = true;
                                        };
                                        if ui.button("🗑").clicked() {
                                            keep = false;
                                        };
                                    });
                                });
                            });
                            keep
                        });
                    });
            });
        });
    }

    fn add_window(&mut self, ui: &mut Ui) {
        let mut style = ui.style_mut();
        style.spacing.button_padding = (16., 8.).into();
        egui::Grid::new("adding_grid")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Type");
                let adding = &mut self.adding_service;
                egui::ComboBox::from_label("Take your pick")
                    .selected_text(format!("{adding:?}"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(adding, Types::Process, "Process");
                        ui.selectable_value(adding, Types::Service, "Service");
                    });
                ui.end_row();
                ui.label("Name");
                ui.text_edit_singleline(&mut self.adding_data.name);
                ui.end_row();
                ui.label("Enabled");
                ui.checkbox(&mut self.adding_data.enabled, "");
                ui.end_row();
                ui.label("Restore");
                ui.checkbox(&mut self.adding_data.restore, "");
                if let Types::Process = self.adding_service {
                    ui.end_row();
                    ui.label("Admin");
                    ui.checkbox(&mut self.adding_data.admin, "");
                }
            });
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                match self.adding_service {
                    Types::Process => self.data.processes.push(Kill {
                        name: self.adding_data.name.clone(),
                        restore: self.adding_data.restore,
                        enabled: self.adding_data.enabled,
                        admin: self.adding_data.admin,
                    }),
                    Types::Service => self.data.services.push(KillService {
                        name: self.adding_data.name.clone(),
                        restore: self.adding_data.restore,
                        enabled: self.adding_data.enabled,
                    }),
                }
                self.adding = false;
            }
            if ui.button("Close").clicked() {
                self.adding = false;
            }
        });
    }
}

impl eframe::App for MyApp {
    fn on_close_event(&mut self) -> bool {
        self.show_action_confirmation = false;
        true
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Ok(prettied) = serde_json::to_string_pretty(&self.data) {
            let mut file = match File::create(*TO_KILL) {
                Ok(file) => file,
                Err(error) => {
                    println!("Error creating file: {error}");
                    return;
                }
            };
            if let Err(error) = file.write_all(prettied.as_bytes()) {
                eprintln!("Error writing to file: {error}");
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                self.process_table(ui);
                ui.vertical_centered_justified(|ui| {
                    let mut style = ui.style_mut();
                    style.spacing.button_padding = (64., 16.).into();

                    if self.show_message {
                        ui.label(&self.result);
                    } else {
                        ui.label("Idle");
                    }

                    if ui.button("Kill").clicked() {
                        self.show_action_confirmation = true;
                        self.chosen_action = Action::Kill;
                    }

                    if ui.button("Restore").clicked() {
                        self.show_action_confirmation = true;
                        self.chosen_action = Action::Restore;
                    }

                    ui.add_space(ui.available_height() - 64.);
                    if ui.button("Add").clicked() {
                        self.adding = true;
                    }
                });
            });
        });

        if self.show_action_confirmation {
            // Show confirmation dialog:
            egui::Window::new(format!("Are you sure you want to {}?", self.chosen_action))
                .collapsible(false)
                .constrain(true)
                .fixed_size((POPAP_W, POPAP_H))
                .anchor(Align2::CENTER_CENTER, (-5., 0.))
                .show(ctx, |ui| self.confirmation_dialog(ui));
        }

        if self.adding {
            // Show adding dialog
            egui::Window::new("Add")
                .collapsible(false)
                .constrain(true)
                .fixed_size((POPAP_W, POPAP_H))
                .anchor(Align2::CENTER_CENTER, (-5., 0.))
                .show(ctx, |ui| self.add_window(ui));
        }

        if let Some(editing) = self.editing_index {
            // Show editing dialog
            egui::Window::new("Editing")
                .collapsible(false)
                .constrain(true)
                .fixed_size((POPAP_W, POPAP_H))
                .anchor(Align2::CENTER_CENTER, (-5., 0.))
                .show(ctx, |ui| self.edit_window(ui, editing));
        }
    }
}
