extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;
#[macro_use]
extern crate enclose;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    console, HtmlCanvasElement, HtmlElement, HtmlInputElement, MouseEvent, WebGlProgram,
    WebGlRenderingContext, WebGlShader, Window,
};

#[wasm_bindgen]
pub fn init() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let state = Rc::new(RefCell::new(State::init(canvas.clone())));

    let mouse_move_callback = Closure::wrap(Box::new(enclose!((state) move |mouse_event| {
        state.borrow_mut().mouse_move(mouse_event);
    })) as Box<FnMut(MouseEvent)>);
    AsRef::<HtmlElement>::as_ref(&canvas)
        .set_onmousemove(Some(mouse_move_callback.as_ref().unchecked_ref()));
    mouse_move_callback.forget();

    let solid_block_radio = document
        .get_element_by_id("solid_block_mode")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let solid_block_callback = Closure::wrap(Box::new(enclose!((state) move || {
        state.borrow_mut().set_mode(Mode::SolidBlock);
    })) as Box<FnMut()>);
    solid_block_radio.set_onclick(Some(solid_block_callback.as_ref().unchecked_ref()));
    solid_block_callback.forget();

    let light_block_radio = document
        .get_element_by_id("light_block_mode")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let light_block_callback = Closure::wrap(Box::new(enclose!((state) move || {
        state.borrow_mut().set_mode(Mode::LightBlock);
    })) as Box<FnMut()>);
    light_block_radio.set_onclick(Some(light_block_callback.as_ref().unchecked_ref()));
    light_block_callback.forget();

    let point_light_radio = document
        .get_element_by_id("point_light_mode")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let point_light_callback = Closure::wrap(Box::new(enclose!((state) move || {
        state.borrow_mut().set_mode(Mode::PointLight);
    })) as Box<FnMut()>);
    point_light_radio.set_onclick(Some(point_light_callback.as_ref().unchecked_ref()));
    point_light_callback.forget();

    let erase_mode_radio = document
        .get_element_by_id("erase_mode")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    let erase_mode_callback = Closure::wrap(Box::new(enclose!((state) move || {
        state.borrow_mut().set_mode(Mode::Erase);
    })) as Box<FnMut()>);
    erase_mode_radio.set_onclick(Some(erase_mode_callback.as_ref().unchecked_ref()));
    erase_mode_callback.forget();

    let angle_input = document
        .get_element_by_id("angle")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let angle_callback = Closure::wrap(Box::new(enclose!((state, angle_input) move || {
        state.borrow_mut().set_angle(angle_input.value().parse::<f32>().unwrap() / 100.0);
    })) as Box<FnMut()>);
    AsRef::<HtmlElement>::as_ref(&angle_input)
        .set_onchange(Some(angle_callback.as_ref().unchecked_ref()));
    angle_callback.forget();

    let pointiness_input = document
        .get_element_by_id("pointiness")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let pointiness_callback = Closure::wrap(Box::new(enclose!((state, pointiness_input) move || {
        state.borrow_mut().set_pointiness(pointiness_input.value().parse::<f32>().unwrap() / 100.0);
    })) as Box<FnMut()>);
    AsRef::<HtmlElement>::as_ref(&pointiness_input)
        .set_onchange(Some(pointiness_callback.as_ref().unchecked_ref()));
    pointiness_callback.forget();

    let clear_button = document
        .get_element_by_id("clear")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    let clear_callback = Closure::wrap(Box::new(enclose!((state) move || {
        state.borrow_mut().clear();
    })) as Box<FnMut()>);
    clear_button.set_onclick(Some(clear_callback.as_ref().unchecked_ref()));
    clear_callback.forget();

    let advance_button = document
        .get_element_by_id("advance")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    let advance_callback = Closure::wrap(Box::new(enclose!((state) move || {
        state.borrow_mut().advance();
    })) as Box<FnMut()>);
    advance_button.set_onclick(Some(advance_callback.as_ref().unchecked_ref()));
    advance_callback.forget();

    let reset_button = document
        .get_element_by_id("reset")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    let reset_callback = Closure::wrap(Box::new(enclose!((state) move || {
        state.borrow_mut().reset();
    })) as Box<FnMut()>);
    reset_button.set_onclick(Some(reset_callback.as_ref().unchecked_ref()));
    reset_callback.forget();

    fn draw(window: Window, state: Rc<RefCell<State>>) {
        state.borrow_mut().draw();

        let callback_window = window.clone();
        let callback = Closure::wrap(Box::new(move |_time| {
            draw(callback_window.clone(), state.clone());
        }) as Box<FnMut(f64)>);
        window
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .unwrap();
        callback.forget();
    }
    draw(window, state);
}

#[derive(Debug)]
enum Mode {
    SolidBlock,
    LightBlock,
    PointLight,
    Erase,
}

struct State {
    context: WebGlRenderingContext,
}

impl State {
    fn init(canvas: web_sys::HtmlCanvasElement) -> State {
        let width = canvas
            .dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .offset_width() as u32;
        let height = canvas
            .dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .offset_height() as u32;

        canvas.set_width(width);
        canvas.set_height(height);

        let context = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();
        context.viewport(0, 0, width as i32, height as i32);

        let vert_shader = compile_shader(
            &context,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
                precision mediump float;

                attribute vec2 a_position;
                attribute vec3 a_color;

                varying vec3 v_color;

                void main() {
                    v_color = a_color;
                    gl_Position = vec4(a_position, 0.0, 1.0);
                }
            "#,
        )
        .unwrap();
        let frag_shader = compile_shader(
            &context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
                precision mediump float;

                varying vec3 v_color;

                void main() {
                    gl_FragColor = vec4(v_color, 1.0);
                }
            "#,
        )
        .unwrap();
        let program = link_program(&context, [vert_shader, frag_shader].iter()).unwrap();
        context.use_program(Some(&program));

        State { context }
    }

    fn draw(&mut self) {
        self.context.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        let vertices = [
            -0.7, -0.7, 1.0, 0.0, 0.0, 0.7, -0.7, 0.0, 1.0, 0.0, 0.0, 0.7, 0.0, 0.0, 1.0,
        ];
        let vert_array =
            js_sys::Float32Array::new(&wasm_bindgen::JsValue::from(vertices.len() as u32));
        for (i, f) in vertices.iter().enumerate() {
            vert_array.fill(*f, i as u32, (i + 1) as u32);
        }

        let buffer = self.context.create_buffer().unwrap();
        self.context
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        self.context.buffer_data_with_opt_array_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&vert_array.buffer()),
            WebGlRenderingContext::STATIC_DRAW,
        );
        self.context.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            5 * 4,
            0 * 4,
        );
        self.context.enable_vertex_attrib_array(0);

        self.context.vertex_attrib_pointer_with_i32(
            1,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            5 * 4,
            2 * 4,
        );
        self.context.enable_vertex_attrib_array(1);

        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.context.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (vertices.len() / 5) as i32,
        );
    }

    fn mouse_move(&mut self, _mouse_event: MouseEvent) {
        console::log_1(&"mouse move event".into());
    }

    fn set_mode(&mut self, mode: Mode) {
        console::log_1(&format!("mode change {:?}", mode).into());
    }

    fn set_angle(&mut self, amt: f32) {
        console::log_1(&format!("set angle {}", amt).into());
    }

    fn set_pointiness(&mut self, amt: f32) {
        console::log_1(&format!("set pointiness {}", amt).into());
    }

    fn clear(&mut self) {
        console::log_1(&"clear button clicked".into());
    }

    fn advance(&mut self) {
        console::log_1(&"advance button clicked".into());
    }

    fn reset(&mut self) {
        console::log_1(&"reset button clicked".into());
    }
}

fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}

fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
    context: &WebGlRenderingContext,
    shaders: T,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    for shader in shaders {
        context.attach_shader(&program, shader)
    }
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program object".into()))
    }
}
