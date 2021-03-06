use druid::widget::{Align, Button, Flex, Label, LensWrap, TextBox};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use futures::prelude::*;
use pharos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::spawn_local;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use ws_stream_wasm::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        fn init_log() {
            use log::Level;
            console_log::init_with_level(Level::Trace).expect("error initializing log");
        }
    } else {
        fn init_log() {
            use log::Level;
            simple_logger::init_with_level(Level::Trace).unwrap();
        }
    }
}

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;

#[derive(Clone, Data, Lens)]
struct ToyState {
    name: String,
    login: LoginState,
}

#[derive(Clone, Data, Lens)]
struct LoginState {
    username: String,
    password: String,
}

// This wrapper function is the primary modification we're making to the vanilla
// hello.rs example.
#[wasm_bindgen]
pub fn wasm_main() {
    // This hook is necessary to get panic messages in the console
    cfg_if::cfg_if! {
        if #[cfg(target_family = "wasm")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        }
    }
    main()
}

pub fn main() {
    init_log();
    log::info!("Starting up...");
    // describe the main window
    //
    // Window title is set in index.html and window size is ignored on the web,
    // so can we leave those off.
    let main_window = WindowDesc::new(build_root_widget());

    // create the initial app state
    let initial_state = ToyState {
        name: "World".into(),

        login: LoginState {
            username: "".into(),
            password: "".into(),
        },
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<ToyState> {
    // a label that will determine its text based on the current app data.
    let label = Label::new(|data: &ToyState, _env: &Env| format!("Hello {}!", data.name));
    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(ToyState::name);

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox)
        .with_child(LensWrap::new(login_widget(), ToyState::login));

    // center the two widgets in the available space
    Align::centered(layout)
}

fn login_widget() -> impl Widget<LoginState> {
    let username = TextBox::new()
        .with_placeholder(LocalizedString::new("username"))
        .lens(LoginState::username);
    let password = TextBox::new()
        .with_placeholder(LocalizedString::new("password"))
        .lens(LoginState::password);
    let login_button =
        Button::new(LocalizedString::new("login")).on_click(|_ctx, data, _env| login(data));

    Flex::row()
        .with_child(username)
        .with_child(password)
        .with_child(login_button)
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn login(login_state: &mut LoginState) {
    spawn_local(async {
        let (mut ws, mut wsio) = WsMeta::connect("ws://127.0.0.1:8000/ws/", None)
            .await
            .expect_throw("assume the connection succeeds");
        wsio.send(WsMessage::Text("login u p".to_string()))
            .await
            .expect_throw("assume login sending succeeds");
        //        let evts = ws.observe(ObserveConfig::default()).expect_throw("bla");
        ws.close().await;

        // Note that since WsMeta::connect resolves to an opened connection, we don't see
        // any Open events here.
        //
        //assert!(evts.next().await.unwrap_throw().is_closing());
        //assert!(evts.next().await.unwrap_throw().is_closed());
    });
}
