use crate::misc::Vector2;
use ggez::event::{KeyCode, MouseButton};
use imgui::*;
use std::collections::HashSet;

#[derive(Copy, Clone, PartialEq)]
pub enum DisplayFlowField {
    Cost,
    Integration,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CursorControl {
    CostDrawing,
    TripSetting,
}

pub struct HighLevelUI {
    pub cursor_control: CursorControl,
    pub flowfield_mode: DisplayFlowField,
    pub flowfield_show_arrow: bool,
    pub compute_live: bool,
    pub compute_step: bool,
    pub set_demo: bool,
    pub step_per_frame: i32,
    pub compute_all: bool,
    pub mouse_pos: Vector2,
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_triggered: HashSet<KeyCode>,
    pub mouse_pressed: HashSet<MouseButton>,
    pub mouse_triggered: HashSet<MouseButton>,
    pub zoom: f32,
    pub zoom_smooth: f32,
    pub cam_pos: Vector2,
    pub cam_pos_smooth: Vector2,
    pub mouse_pos_camera: Vector2,
    pub last_compute_ms: u128,
    pub auto_delete: bool,

    pub full_pathfinding: Vec<(bool, String)>,
    pub number_of_agent: usize,
}

impl HighLevelUI {
    pub fn new() -> HighLevelUI {
        HighLevelUI {
            cursor_control: CursorControl::CostDrawing,
            flowfield_mode: DisplayFlowField::Cost,
            flowfield_show_arrow: false,
            compute_live: true,
            compute_step: false,
            set_demo: false,
            step_per_frame: 2,
            compute_all: true,
            mouse_pos: Vector2::new(0.0, 0.0),
            keys_pressed: HashSet::new(),
            keys_triggered: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_triggered: HashSet::new(),
            zoom: 1.0,
            zoom_smooth: 1.0,
            cam_pos: Vector2::new(-250.0, -400.0),
            cam_pos_smooth: Vector2::new(0.0, 0.0),
            mouse_pos_camera: Vector2::new(0.0, 0.0),
            last_compute_ms: 0,
            auto_delete: true,
            full_pathfinding: Vec::new(),
            number_of_agent: 0,
        }
    }

    pub fn mouse_trigger(&mut self, mb: MouseButton) {
        self.mouse_triggered.insert(mb);
        self.mouse_pressed.insert(mb);
    }

    pub fn key_trigger(&mut self, mb: KeyCode) {
        self.keys_pressed.insert(mb);
        self.keys_triggered.insert(mb);
    }

    pub fn get_mouse_pressed_or_triggered(&self) -> HashSet<MouseButton> {
        self.mouse_pressed
            .union(&self.mouse_triggered)
            .copied()
            .collect()
    }
    pub fn reset_trigger(&mut self) {
        self.mouse_triggered.clear();
        self.keys_triggered.clear();
    }

    pub fn draw(&mut self, ui: &imgui::Ui) {
        {
            // Window
            Window::new(im_str!("Rust field"))
            .size([300.0, 400.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                                ui.text(im_str!("Try the "));
                                ui.same_line(0.0);
                                if ui.small_button(im_str!("Demo")) {
                                    self.set_demo=  true;
                                };
                                ui.text(im_str!("Draw a maze"));
                                ui.text(im_str!("Then check the 'Trip setting'"));

                                ui.separator();

                                ui.text(im_str!("Control: "));
                                ui.radio_button(im_str!("Cost drawing"),&mut self.cursor_control,CursorControl::CostDrawing);
                                ui.same_line(0.0);
                                ui.radio_button(im_str!("Trip setting"),&mut self.cursor_control,CursorControl::TripSetting);

                                match self.cursor_control{
                                    CursorControl::CostDrawing =>{
                                        ui.bullet_text(im_str!("Left click : Wall"));
                                        ui.bullet_text(im_str!("Right click : Erase"));
                                        ui.bullet_text(im_str!("Middle click : Reset"));
                                    }
                                    CursorControl::TripSetting =>{
                                        ui.bullet_text(im_str!("Left click : Place start"));
                                        ui.bullet_text(im_str!("Right click : Place end"));
                                        ui.bullet_text(im_str!("Middle click : Reset path"));
                                    }
                                }
                                ui.bullet_text(im_str!("ZQSD : Pan camera"));
                                ui.bullet_text(im_str!("Scroll : Zoom camera"));
                                ui.bullet_text(im_str!("Space : Spawn 250 agents"));
                                ui.bullet_text(im_str!("Delete : Mass Murder"));
                                ui.separator();
                                ui.text(im_str!("Display: "));
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

                                ui.separator();
                                ui.text(im_str!("Computations: "));
                                ui.checkbox(im_str!("Auto delete old path"), &mut self.auto_delete);
                                ui.checkbox(im_str!("Compute all instantly"), &mut self.compute_all);
                                ui.same_line(0.0);
                                ui.text(im_str!(
                                    "/ {} ms",
                                    self.last_compute_ms
                                ));


                                if !self.compute_all {
                                    ui.checkbox(im_str!("Compute live"), &mut self.compute_live);
                                }

                                if !self.compute_all  && self.compute_live{
                                    imgui::Slider::new(im_str!("step per frame "),0..=100).build(ui, &mut self.step_per_frame,);
                                }

                                if !self.compute_live && !self.compute_all {
                                    if ui.small_button(im_str!("Compute step")) {
                                        self.compute_step = true;
                                    }
                                }
                                ui.separator();

                                ui.text(im_str!("Path list: "));
                                for (index,e) in self.full_pathfinding.iter_mut().enumerate() {
                                    ui.text(im_str!(
                                    "path#{}",
                                    index
                                ));
                                    ui.same_line(0.0);
                                    if ui.small_button(im_str!("Delete##{}",index).as_ref()) {
                                        e.0 = true;
                                    }
                                }

                                ui.separator();
                                ui.text(im_str!("Agents: {}", self.number_of_agent));


                            });
        }
    }
}
