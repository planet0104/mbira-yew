use std::rc::Rc;
use std::time::Duration;

use tabs::Phenotype;
use tabs::Tab;
use tabs::parse_tabs;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::Callback;
use yew::ChangeData;
use yew::services::DialogService;
use yew::services::TimeoutService;
use yew::services::timeout::TimeoutTask;
use yew::services::{console::ConsoleService, RenderService, Task};
use yew::{html, Component, ComponentLink, Html, NodeRef, ShouldRender};
use yew_styles::forms::form_select::FormSelect;
use yew_styles::styles::Size;
mod tools;
mod tabs;

pub struct Model {
    root_ref: NodeRef,
    canvas: Option<HtmlCanvasElement>,
    context2d: Option<CanvasRenderingContext2d>,
    link: ComponentLink<Self>,
    node_ref: NodeRef,
    render_loop: Option<Box<dyn Task>>,
    error_mssages: String,
    piano_width: f64,
    piano_height: f64,
    window_width: f64,
    window_height: f64,
    color_white: JsValue,
    color_black: JsValue,
    color_gray: JsValue,
    color_light_gray: JsValue,
    full_screen: bool,
    timeout_task: Option<TimeoutTask>,
    
    /// 当前播放的歌曲
    current_tab: Option<Vec<Tab>>,
    /// 当前播放的音符
    current_tab_index: Option<usize>,
    /// 当前绘制的音符
    phenotypes: Vec<Phenotype>,
    /// 每个键对应的x坐标和宽度
    tab_positions: Vec<(f64, f64)>,
    next_tab_time: f64,
    time_delay: i32,
    start_time: f64,
    sound_name: String,
    bpm: f64,
}

pub enum Msg {
    Render(f64),
    Success(String),
    Error(String),
    Sound(String),
    OnSizeChange(ChangeData),
    OnWindowSizeChange,
    OnCanvasClick,
    OnTabSelect(String),
    None,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {

        let window = window().unwrap();
        let width:f64 = window.inner_width().unwrap().as_f64().unwrap();
        let height:f64 = window.inner_height().unwrap().as_f64().unwrap();
        ConsoleService::log(&format!("窗口大小 {}x{}", width, height));

        Model {
            canvas: None,
            context2d: None,
            link,
            node_ref: NodeRef::default(),
            root_ref: NodeRef::default(),
            render_loop: None,
            error_mssages: String::new(),
            piano_width: width,
            piano_height: height,
            window_width: width,
            window_height: height,
            color_white: JsValue::from_str("white"),
            color_black: JsValue::from_str("black"),
            color_gray: JsValue::from_str("#3b403b"),
            color_light_gray: JsValue::from_str("#aaa"),
            full_screen: false,
            timeout_task: None,

            current_tab: None,
            current_tab_index: None,
            phenotypes: vec![],
            tab_positions: vec![(0., 0.); tabs::KEYS.len()],
            next_tab_time: 0.,
            time_delay: 3,
            start_time: 0.,
            sound_name: String::new(),
            bpm: 50.,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        // 初始化绘图
        if first_render {
            // 渲染完成，保存canvas和context2d对象

            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
            
            canvas.set_width(self.window_width as u32);
            canvas.set_height(self.window_height as u32);
            
            let context2d = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            
            self.canvas = Some(canvas);
            self.context2d = Some(context2d);

            // 请求动画帧的回调传递一个时间值，该时间值可用于渲染动画
            let render_frame = self.link.callback(Msg::Render);
            let handle = RenderService::request_animation_frame(render_frame);

            // 必须存储对句柄的引用，否则将丢弃该句柄，并且不会进行渲染。
            self.render_loop = Some(Box::new(handle));

            if let Some(dp) = window().unwrap().local_storage().unwrap().unwrap().get_item("dp").unwrap(){
                self.link.send_message(Msg::OnSizeChange(ChangeData::Value(dp)));
            }
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::OnCanvasClick =>{
                let link_clone = self.link.clone();
                if self.full_screen{
                    window().unwrap().document().unwrap().exit_fullscreen();
                    self.full_screen = false;
                    self.timeout_task = Some(TimeoutService::spawn(Duration::from_millis(300), Callback::Callback(Rc::new(move |_|{
                        link_clone.send_message(Msg::OnWindowSizeChange);
                    }))));
                }else{
                    window().unwrap().document().unwrap().body().unwrap().request_fullscreen().unwrap();
                    // self.canvas.as_ref().unwrap().request_fullscreen().unwrap();
                    self.full_screen = true;
                    self.timeout_task = Some(TimeoutService::spawn(Duration::from_millis(300), Callback::Callback(Rc::new(move |_|{
                        link_clone.send_message(Msg::OnWindowSizeChange);
                    }))));
                }
                false
            }
            Msg::OnTabSelect(name) => {
                self.sound_name = String::from(&name);
                if name == "但愿人长久"{
                    self.bpm = 88.;
                    self.init_tabs(tabs::DAN_YUAN_REN_CHANG_JIU);
                }else if name == "蝶恋"{
                    self.bpm = 80.;
                    self.init_tabs(tabs::DIELIAN);
                }else if name == "梁祝"{
                    self.bpm = 56.;
                    self.init_tabs(tabs::LIANG_ZHU);
                }else if name == "逍遥叹-胡歌"{
                    self.bpm = 68.;
                    self.init_tabs(tabs::XIAOYAOTAN);
                }
                true
            }
            Msg::OnWindowSizeChange => {
                let window = window().unwrap();
                let width:f64 = window.inner_width().unwrap().as_f64().unwrap();
                let height:f64 = window.inner_height().unwrap().as_f64().unwrap();
                ConsoleService::log(&format!("窗口大小 {}x{}", width, height));
                self.window_width = width;
                self.window_height = height;
                self.canvas.as_ref().unwrap().set_width(width as u32);
                self.canvas.as_ref().unwrap().set_height(height as u32);
                true
            }
            Msg::None => false,
            Msg::OnSizeChange(v) => {
                if let ChangeData::Value(v) = v{
                    let dp = v.parse::<f64>().unwrap();
                    self.piano_width = self.window_width * (1000.-dp)/1000.;
                    //存储至本地
                    window().unwrap().local_storage().unwrap().unwrap().set_item("dp", &v).unwrap();
                }
                true
            }
            Msg::Sound(tone) => {
                ConsoleService::log(&format!("tone: {}", tone));
                self.error_mssages.push_str(&format!("{} ", tone));
                // DialogService::alert(&format!("tone: {:?}", tone));
                true
            }
            Msg::Error(err) => {
                ConsoleService::error(&format!("estimator 错误: {}", err));
                self.error_mssages
                    .push_str(&format!("estimator 错误: {}\r\n", err));
                // DialogService::alert(&format!("estimator 错误: {:?}", err));
                true
            }
            Msg::Success(msg) => {
                ConsoleService::log(&format!("estimator {}", msg));
                // DialogService::alert(&format!("estimator {}", msg));
                self.error_mssages
                    .push_str(&format!("estimator {}\r\n", msg));
                true
            }
            Msg::Render(timestamp) => {
                self.render(timestamp);
                false
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div ref={ self.root_ref.clone() }>
                <canvas onclick=self.link.callback(|_| Msg::OnCanvasClick) ref={self.node_ref.clone()} />
                <div class="buttons">
                    <input onchange=self.link.callback(|v| Msg::OnSizeChange(v) ) type="range"  min="-800" max="800" />
                    <FormSelect
                        select_size=Size::Big
                        onchange_signal = self.link.callback(|e: ChangeData|
                            match e {
                                ChangeData::Select(element) => {
                                    let value = element.value();
                                    Msg::OnTabSelect(value)
                                },
                                _ => unreachable!(),
                            }
                        )
                        options=html!{
                            <>
                                <option value="请选择">{"请选择"}</option>
                                <option value="逍遥叹-胡歌">{"逍遥叹-胡歌"}</option>
                                <option value="梁祝">{"梁祝"}</option>
                                <option value="但愿人长久">{"但愿人长久"}</option>
                                <option value="蝶恋">{"蝶恋"}</option>
                            </>
                        }
                    />
                </div>
            </div>
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}

impl Model {
    fn render(&mut self, timestamp: f64) {
        let ctx = self.context2d.as_ref().unwrap();

        let (window_width, window_height) = (self.window_width, self.window_height);

        ctx.clear_rect(0., 0., window_width, window_height);
        
        let mut x = (window_width - self.piano_width)/2.;
        let y = 40.;
        //平均绘制17个条
        //94px:50px 94x17=1598(0.66638) 50x16=800(0.333) total=2398
        let w1 = (0.66638 * self.piano_width)/17.;
        let radius = w1/2.;
        let w2 = 0.5319 * w1;
        ctx.set_font(&format!("{}px Arial", (w1*0.57) as i32));
        //第一个琴键
        ctx.set_fill_style(&self.color_gray);
        tools::draw_round_rect(ctx, x, y, w1, self.window_height, radius, true, false);

        ctx.set_fill_style(&self.color_light_gray);
        ctx.fill_text(tabs::KEYS[0], x+w1/7., y+w1*0.8).unwrap();
        self.tab_positions[0].0 = x;
        self.tab_positions[0].1 = w1;

        for i in 0..16{
            x += w1;
            ctx.set_fill_style(&self.color_black);
            ctx.fill_rect(x, y, w2, self.window_height);
            x += w2;
            ctx.set_fill_style(&self.color_gray);
            tools::draw_round_rect(ctx, x, y, w1, self.window_height, radius, true, false);
            ctx.set_fill_style(&self.color_light_gray);
            ctx.fill_text(tabs::KEYS[i+1], x+w1/7., y+w1*0.8).unwrap();
            self.tab_positions[i+1].0 = x;
            self.tab_positions[i+1].1 = w1;
        }
         
        if self.start_time == 0.{
            self.start_time = timestamp;
        }

        if let Some(tabs) = self.current_tab.as_ref(){
            self.time_delay = 3 - ((timestamp-self.start_time)/1000.) as i32;
            if self.current_tab_index.is_none(){
                if self.time_delay < 0{
                    self.current_tab_index = Some(0);
                    let tab = &tabs[0];
                    for tone in &tab.tones{
                        let tab_index = tone.1;
                        let (x, width) = self.tab_positions[tab_index];
                        //添加一个向下移动的音符
                        self.phenotypes.push(Phenotype{
                            x, y, width, height: width/2., color: JsValue::from_str("red"),
                            word: tab.word.clone(),
                            tone: tone.0.clone(),
                        })
                    }
                    let dur = (1000.*60.)/self.bpm*(4./tabs[0].time);
                    ConsoleService::log(&format!("dur={}", dur));
                    self.next_tab_time =  timestamp + dur;   
                }else{
                    ctx.set_fill_style(&self.color_white);
                    ctx.set_font(&format!("{}pt Arial", 100));
                    ctx.fill_text(&format!("{}", self.time_delay), window_width/2.1, window_height/2.).unwrap();
                    ConsoleService::log(&format!("self.time_delay={}", self.time_delay));
                }
            }else{
                if timestamp > self.next_tab_time{
                    let mut current_tab_index = self.current_tab_index.unwrap();
                    current_tab_index += 1;
                    if current_tab_index < tabs.len(){
                        let tab = &tabs[current_tab_index];
                        for tone in &tab.tones{
                            let tab_index = tone.1;
                            let (x, width) = self.tab_positions[tab_index];
                            //添加一个向下移动的音符
                            self.phenotypes.push(Phenotype{
                                x, y, width, height: width/2., color: JsValue::from_str("red"),
                                word: tab.word.clone(),
                                tone: tone.0.clone(),
                            })
                        }
                        self.current_tab_index = Some(current_tab_index);
                        self.next_tab_time =  timestamp + (1000.*60.)/self.bpm*(4./tabs[current_tab_index].time);
                    }else{
                        self.current_tab_index = None;
                        self.time_delay = 3;
                        self.start_time = timestamp;
                    }
                }
            }
        }
 
        self.phenotypes.retain(|p| p.y < window_height );
        for p in &mut self.phenotypes{
            p.update();
            p.render(ctx);
        }

        //绘制歌曲名称
        ctx.set_font("30pt Arial");
        ctx.set_fill_style(&JsValue::from_str("rgba(255,255,255, 0.7)"));
        ctx.fill_text(&self.sound_name, (window_width - self.piano_width)/2.+20., 120.).unwrap();

        let render_frame = self.link.callback(Msg::Render);
        let handle = RenderService::request_animation_frame(render_frame);

        // 必须保留对新句柄的引用才能运行下一个渲染。
        self.render_loop = Some(Box::new(handle));
    }

    fn init_tabs(&mut self, tabs:&str){
        let tabs = parse_tabs(tabs);
        if tabs.is_none() {
            DialogService::alert("琴谱解析失败！");
            return;
        }
        let tabs = tabs.unwrap();
        self.current_tab = Some(tabs);
        self.current_tab_index = None;
        self.time_delay = 3;
        self.start_time = 0.;
    }
}
#[wasm_bindgen(start)]
pub fn run_app() {
    yew::start_app::<Model>();
}
