use crate::misc::Vector2;
use ggez::event::{KeyCode, MouseButton};
use imgui::*;
use std::collections::HashSet;

#[derive(Copy, Clone, PartialEq)]
pub enum DisplayFlowField {
    Cost,
    Integration,
}

pub struct UI {
    pub flowfield_mode: DisplayFlowField,
    pub flowfield_show_arrow: bool,
    pub compute_live: bool,
    pub compute_step: bool,
    pub compute_all: bool,
    pub mouse_pos: Vector2,
    pub keys_pressed: HashSet<KeyCode>,
    pub mouse_pressed: HashSet<MouseButton>,
    pub zoom: f32,
    pub zoom_smooth: f32,
    pub cam_pos: Vector2,
    pub cam_pos_smooth: Vector2,
    pub mouse_pos_camera: Vector2,
}

impl UI {
    pub fn new() -> UI {
        UI {
            flowfield_mode: DisplayFlowField::Cost,
            flowfield_show_arrow: false,
            compute_live: false,
            compute_step: false,
            compute_all: true,
            mouse_pos: Vector2::new(0.0, 0.0),
            keys_pressed: HashSet::new(),
            mouse_pressed: HashSet::new(),
            zoom: 1.0,
            zoom_smooth: 1.0,
            cam_pos: Vector2::new(-250.0, -400.0),
            cam_pos_smooth: Vector2::new(0.0, 0.0),
            mouse_pos_camera: Vector2::new(0.0, 0.0),
        }
    }

    pub fn draw(&mut self, ui: &imgui::Ui) {
        {
            // Window
            ui.window(im_str!("Rust field"))
                .size([300.0, 400.0], imgui::Condition::FirstUseEver)
                .position([400.0, 100.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    match self.flowfield_mode{
                        DisplayFlowField::Cost => {
                            ui.text(im_str!("Draw a maze"));
                            ui.bullet_text(im_str!("Left click : Wall"));
                            ui.bullet_text(im_str!("Right click : Erase"));
                            ui.bullet_text(im_str!("Middle click : Reset"));
                            ui.text(im_str!("Then check the 'integration' display"));
                        }
                        DisplayFlowField::Integration => {
                            ui.text(im_str!("Set a destination"));
                            ui.bullet_text(im_str!("Left click : place target"));
                            ui.text(im_str!("Play with the settings below"));
                            ui.text(im_str!(""));
                            ui.text(im_str!(""));
                        }
                    }

                    ui.separator();
                    ui.text(im_str!("Camera control: ZSQD/Scroll"));
                    ui.separator();
                    ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        self.mouse_pos_camera.x,
                        self.mouse_pos_camera.y
                    ));
                    ui.separator();

                    ui.text(im_str!("Display: "));
                    ui.radio_button(im_str!("Cost field"),&mut self.flowfield_mode,DisplayFlowField::Cost);
                    ui.radio_button(im_str!("Integration field"),&mut self.flowfield_mode,DisplayFlowField::Integration);

                    if ui.checkbox(im_str!("Show flow arrows"), &mut self.flowfield_show_arrow) {
                        println!("check changed");
                        println!("to {}", self.flowfield_show_arrow);
                    }

                    if ui.is_item_hovered(){
                        ui.tooltip(||
                                       {
                                           let tok = ui.push_text_wrap_pos(ui.get_cursor_pos()[0]+ 150.0);
                                           ui.text(im_str!("Shows an arrow on each cell of the map indicating the shortest way to the target"));
                                           std::mem::drop(tok);
                                       }
                        );
                    }

                    ui.checkbox(im_str!("Compute all"), &mut self.compute_all);

                    if !self.compute_all {
                        ui.checkbox(im_str!("Compute live"), &mut self.compute_live);
                    }

                    if !self.compute_live && !self.compute_all {
                        if ui.small_button(im_str!("Compute step")) {
                            self.compute_step = true;
                        }
                    }
                });
        }
    }
}
