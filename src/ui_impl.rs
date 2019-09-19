use crate::misc::Vector2;
use ggez::event::{KeyCode, KeyMods, MouseButton};
use imgui::*;
use std::collections::HashSet;

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
                    ui.text(im_str!("Draw a maze"));
                    ui.text(im_str!("Left click : Wall"));
                    ui.text(im_str!("Right click : Erase"));
                    ui.text(im_str!("Then click on the Display button"));
                    ui.text(im_str!("Then click anywhere on your maze"));
                    ui.text(im_str!("to generate the flow field"));
                    ui.text(im_str!("leading to where you clicked"));

                    ui.separator();
                    ui.text(im_str!("Camera control: ZSQD/Scroll"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));

                    ui.text(im_str!(
                        "Mouse Position Cam: ({:.1},{:.1})",
                        self.mouse_pos_camera.x,
                        self.mouse_pos_camera.y
                    ));
                    ui.separator();

                    ui.text(im_str!("Display: "));
                    ui.same_line(0.0);

                    let btn_text = match self.flowfield_mode {
                        DisplayFlowField::Cost => "Cost",
                        DisplayFlowField::Integration => "Integration",
                    };
                    let next = match self.flowfield_mode {
                        DisplayFlowField::Cost => DisplayFlowField::Integration,
                        DisplayFlowField::Integration => DisplayFlowField::Cost,
                    };

                    if ui.small_button(&ImString::new(btn_text)) {
                        self.flowfield_mode = next;
                    }

                    if ui.checkbox(im_str!("Show flow arrows"), &mut self.flowfield_show_arrow) {
                        println!("check changed");
                        println!("to {}", self.flowfield_show_arrow);
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
