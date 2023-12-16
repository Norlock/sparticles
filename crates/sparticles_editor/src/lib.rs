pub use crate::pa_widgets::EditorWidgets;
use async_std::task;
use menu::{
    camera_performance::CameraPerformanceMenu,
    declarations::MenuCtx,
    emitter::{EmitterMenu, Tab},
    import::ImportMenu,
    none::NoneMenu,
    post_fx::PostFxMenu,
    MenuWidget,
};
use sparticles_app::{
    animations::{
        ColorAnimation, DiffusionAnimation, ForceAnimation, GravityAnimation, StrayAnimation,
        SwayAnimation,
    },
    fx::{blur::BlurFx, BloomFx, ColorFx},
    gui::egui::{load::SizedTexture, *},
    gui::{
        egui::{self},
        winit::event::{ElementState, KeyboardInput, VirtualKeyCode},
    },
    model::{
        events::ViewIOEvent, EmitterSettings, EmitterUniform, GfxState, SparEvents, SparState,
    },
    profiler::GpuTimerScopeResult,
    texture::IconTexture,
    traits::{EmitterAnimation, ParticleAnimation, PostFx, WidgetBuilder},
    util::ListAction,
    wgpu::{self, CommandEncoder},
};
use std::{
    any::TypeId,
    collections::HashMap,
    path::{Path, PathBuf},
};

pub mod em_widgets;
pub mod fx_widgets;
pub mod menu;
pub mod pa_widgets;

pub type PAWidgetPtr = Box<dyn Fn(&mut EditorData, &mut Box<dyn ParticleAnimation>, &mut Ui)>;
pub type EMWidgetPtr = Box<dyn Fn(&mut EditorData, &mut Box<dyn EmitterAnimation>, &mut Ui)>;
pub type FXWidgetPtr = Box<dyn Fn(&mut EditorData, &mut Box<dyn PostFx>, &mut Ui)>;

pub struct Editor {
    pub data: EditorData,
    pub dyn_widgets: DynamicWidgets,

    /// Menu widgets
    pub menus: Vec<Box<dyn MenuWidget>>,
}

pub struct DynamicWidgets {
    /// Particle animation widgets
    pub pa_widgets: HashMap<TypeId, PAWidgetPtr>,

    /// Emitter animation widgets
    pub em_widgets: HashMap<TypeId, EMWidgetPtr>,

    /// Post fx animation widgets
    pub fx_widgets: HashMap<TypeId, FXWidgetPtr>,
}

pub struct EditorData {
    new_emitter_tag: String,
    profiling_results: Vec<GpuTimerScopeResult>,
    selected_emitter_idx: usize,
    selected_menu_idx: usize,

    fps_text: String,
    frame_time_text: String,
    cpu_time_text: String,
    total_elapsed_text: String,
    particle_count_text: String,
    icon_textures: HashMap<String, TextureId>,
    selected_tab: Tab,
    selected_new_par_anim: usize,
    selected_new_em_anim: usize,
    selected_new_post_fx: usize,

    //performance_event: Option<DisplayEvent>,
    //display_event: Option<DisplayEvent>,
    pub emitter_settings: Option<EmitterSettings>,
    pub model_files: Vec<PathBuf>,
}

const CHEVRON_UP_ID: &str = "chevron-up";
const CHEVRON_DOWN_ID: &str = "chevron-down";
const TRASH_ID: &str = "trash";
const MENU_ID: &str = "menu";
pub const WINDOW_MARGIN: f32 = 10.;

impl WidgetBuilder for Editor {
    fn id(&self) -> &'static str {
        "editor"
    }

    fn process_input(
        &mut self,
        events: &mut SparEvents,
        input: &KeyboardInput,
        shift_pressed: bool,
    ) -> bool {
        if input.state == ElementState::Pressed {
            return false;
        }

        let data = &mut self.data;

        let keycode = input.virtual_keycode.unwrap_or(VirtualKeyCode::Return);

        match keycode {
            VirtualKeyCode::T if shift_pressed => {
                events.io_view = Some(ViewIOEvent::Subtract);
            }
            VirtualKeyCode::T if !shift_pressed => {
                events.io_view = Some(ViewIOEvent::Add);
            }
            VirtualKeyCode::Key1 => data.selected_menu_idx = 1,
            VirtualKeyCode::Key2 => data.selected_menu_idx = 2,
            VirtualKeyCode::Key3 => data.selected_menu_idx = 3,
            VirtualKeyCode::Key4 => data.selected_menu_idx = 4,
            VirtualKeyCode::Key0 => data.selected_menu_idx = 0,
            //VirtualKeyCode::C => gui.display_event.set(DisplayEvent::ToggleCollapse),
            //VirtualKeyCode::P => gui.performance_event.set(DisplayEvent::ToggleCollapse),
            VirtualKeyCode::F => events.toggle_play = true,
            _ => return false,
        }

        true
    }
}

impl Editor {
    pub fn draw_gui(
        &mut self,
        state: &mut SparState,
        events: &mut SparEvents,
        encoder: &mut CommandEncoder,
    ) {
        let ctx = &state.egui_ctx();

        Window::new("Menu")
            .collapsible(false)
            .anchor(Align2::RIGHT_TOP, [-10., 10.])
            .resizable(false)
            .title_bar(false)
            .frame(Frame {
                fill: Color32::GRAY,
                inner_margin: 2f32.into(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let data = &mut self.data;

                ComboBox::from_id_source("select-menu")
                    .width(200.)
                    .show_index(ui, &mut data.selected_menu_idx, self.menus.len(), |i| {
                        RichText::new(format!("{}: {}", i, self.menus[i].title())).size(18.)
                    });
            });

        let idx = self.data.selected_menu_idx;

        let mut menu_ctx = MenuCtx {
            ctx,
            dyn_widgets: &mut self.dyn_widgets,
            emitter_data: &mut self.data,
            state,
            events,
            encoder,
        };

        self.menus[idx].draw_ui(&mut menu_ctx);
    }

    pub fn create_label(ui: &mut Ui, text: impl Into<String>) {
        ui.label(RichText::new(text).color(Color32::WHITE));
        ui.add_space(5.0);
    }

    pub fn create_icons(gfx_state: &mut GfxState) -> HashMap<String, TextureId> {
        let device = &gfx_state.device;
        let queue = &gfx_state.queue;
        let renderer = &mut gfx_state.renderer;

        let mut textures = HashMap::new();

        let mut create_tex = |filename: &str, tag: &str| {
            let icon_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/assets/icons")
                .join(filename);

            let path_str = icon_path
                .to_str()
                .unwrap_or_else(|| panic!("File doesn't exist: {}", filename));

            let view = IconTexture::create_view(device, queue, path_str);

            let texture_id =
                renderer.register_native_texture(device, &view, wgpu::FilterMode::Nearest);

            textures.insert(tag.to_string(), texture_id);
        };

        create_tex("chevron-up.png", CHEVRON_UP_ID);
        create_tex("chevron-down.png", CHEVRON_DOWN_ID);
        create_tex("trash.png", TRASH_ID);
        create_tex("menu.png", MENU_ID);

        textures
    }

    pub fn new(state: &mut SparState, model_dir: PathBuf) -> Self {
        let gfx = &mut task::block_on(state.gfx.write());
        //let texture_paths = Persistence::import_textures().unwrap();
        let icon_textures = Self::create_icons(gfx);
        let mut pa_widgets: HashMap<TypeId, PAWidgetPtr> = HashMap::new();
        let mut em_widgets: HashMap<TypeId, EMWidgetPtr> = HashMap::new();
        let mut fx_widgets: HashMap<TypeId, FXWidgetPtr> = HashMap::new();

        pa_widgets.insert(
            TypeId::of::<ColorAnimation>(),
            Box::new(EditorWidgets::color_anim),
        );

        pa_widgets.insert(
            TypeId::of::<ForceAnimation>(),
            Box::new(EditorWidgets::force_anim),
        );

        pa_widgets.insert(
            TypeId::of::<GravityAnimation>(),
            Box::new(EditorWidgets::gravity_anim),
        );

        pa_widgets.insert(
            TypeId::of::<StrayAnimation>(),
            Box::new(EditorWidgets::stray_anim),
        );

        em_widgets.insert(
            TypeId::of::<SwayAnimation>(),
            Box::new(EditorWidgets::sway_anim),
        );

        em_widgets.insert(
            TypeId::of::<DiffusionAnimation>(),
            Box::new(EditorWidgets::diffusion_anim),
        );

        fx_widgets.insert(TypeId::of::<BloomFx>(), Box::new(EditorWidgets::bloom_fx));
        fx_widgets.insert(TypeId::of::<BlurFx>(), Box::new(EditorWidgets::blur_fx));
        fx_widgets.insert(TypeId::of::<ColorFx>(), Box::new(EditorWidgets::color_fx));

        let mut model_files = vec![];

        match model_dir.read_dir() {
            Ok(dir) => {
                for item in dir.into_iter() {
                    match item {
                        Ok(dir) => {
                            if let Some(extension) = dir.path().extension() {
                                if extension.to_os_string() == *"glb"
                                    || extension.to_os_string() == *"gltf"
                                {
                                    model_files.push(dir.path());
                                }
                            }
                        }
                        Err(err) => {
                            println!("{:?}", err);
                        }
                    }
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }

        let data = EditorData {
            frame_time_text: "".to_string(),
            cpu_time_text: "".to_string(),
            fps_text: "".to_string(),
            total_elapsed_text: "".to_string(),
            particle_count_text: "".to_string(),
            selected_tab: Tab::EmitterSettings,
            selected_menu_idx: 0,
            selected_emitter_idx: 0,
            selected_new_par_anim: 0,
            selected_new_em_anim: 0,
            selected_new_post_fx: 0,
            icon_textures,
            new_emitter_tag: "".to_string(),
            profiling_results: Vec::new(),
            //display_event: None,
            //performance_event: None,
            emitter_settings: None,
            model_files,
        };

        let menus: Vec<Box<dyn MenuWidget>> = vec![
            Box::new(NoneMenu),
            Box::new(EmitterMenu),
            Box::new(ImportMenu),
            Box::new(PostFxMenu),
            Box::new(CameraPerformanceMenu),
        ];

        let dyn_widgets = DynamicWidgets {
            pa_widgets,
            em_widgets,
            fx_widgets,
        };

        Self {
            data,
            menus,
            dyn_widgets,
        }
    }

    pub fn create_degree_slider(ui: &mut Ui, val: &mut f32, str: &str) {
        ui.add(Slider::new(val, 0.0..=360.).text(str));
    }

    fn create_drag_value(ui: &mut Ui, val: &mut f32) {
        ui.add(
            egui::DragValue::new(val)
                .clamp_range(0f64..=f64::MAX)
                .speed(0.1),
        );
    }
}

impl EditorData {
    pub fn create_title(&self, ui: &mut Ui, str: &str) {
        ui.label(RichText::new(str).color(Color32::WHITE).size(16.0));
        ui.add_space(5.0);
    }

    pub fn sync_emitter_settings(&mut self, uniform: &EmitterUniform) {
        if let Some(emitter_settings) = &mut self.emitter_settings {
            if uniform.id != emitter_settings.id {
                *emitter_settings = uniform.create_settings();
            }
        } else {
            self.emitter_settings = Some(uniform.create_settings());
        }
    }

    /// Creates list item header
    pub fn create_li_header(&self, ui: &mut Ui, title: &str) -> ListAction {
        let mut selected_action = ListAction::None;

        ui.horizontal_top(|ui| {
            self.create_title(ui, title);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                let trash_id = self
                    .icon_textures
                    .get(TRASH_ID)
                    .expect("Trash icon doesn't exist");

                let trash_img = SizedTexture::new(*trash_id, [16., 16.]);
                if ui.add(ImageButton::new(trash_img)).clicked() {
                    selected_action = ListAction::Delete;
                };

                let up_id = self
                    .icon_textures
                    .get(CHEVRON_UP_ID)
                    .expect("Chevron up icon doesn't exist");

                let up_img = SizedTexture::new(*up_id, [16., 16.]);
                if ui.add(ImageButton::new(up_img)).clicked() {
                    selected_action = ListAction::MoveUp;
                };

                let down_id = self
                    .icon_textures
                    .get(CHEVRON_DOWN_ID)
                    .expect("Chevron down icon doesn't exist");

                let down_img = SizedTexture::new(*down_id, [16., 16.]);
                if ui.add(ImageButton::new(down_img)).clicked() {
                    selected_action = ListAction::MoveDown;
                };
            });
        });

        selected_action
    }
}

pub trait IntoRichText {
    fn rich_text(&self) -> RichText;
}

impl IntoRichText for PathBuf {
    fn rich_text(&self) -> RichText {
        RichText::new(self.file_name().unwrap().to_str().unwrap())
    }
}
