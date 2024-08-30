#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::str::FromStr;
use freya::prelude::*;
use dioxus_logger::tracing::{debug, Level};
use planner_app::ViewModel;
use crate::app_core::CoreService;
use dioxus_sdk::{
    i18n::{
        use_i18,
        use_init_i18n,
        Language,
    },
    translate,
};
use unic_langid::LanguageIdentifier;

use dioxus_router::prelude::{Outlet, Routable, Router};

mod app_core;

static LANGUAGES: &[LanguagePair] = &[
    LanguagePair { code: "en-US", name: "English (United-States)" },
    LanguagePair { code: "es-ES", name: "Español (España)" },
];

#[derive(Clone)]
struct LanguagePair {
    code: &'static str,
    name: &'static str,
}

fn use_init_languages() {    
    let first_language: &LanguagePair = LANGUAGES.first().unwrap();

    let first_language_identifier: LanguageIdentifier = first_language.code.parse().unwrap();
    
    use_init_i18n(first_language_identifier.clone(), first_language_identifier, || {
        LANGUAGES.iter().map(|LanguagePair { code, name: _name }|{
            match *code {
                "en-US" => Language::from_str(EN_US).unwrap(),
                "es-ES" => Language::from_str(ES_ES).unwrap(),
                _ => panic!()
            }
        }).collect()
    });
    
}

fn app() -> Element {

    use_init_languages();

    rsx!(
        rect {
           font_family: "Arimo Nerd",
            Router::<Route> {}
        }
    )
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppSidebar)]
    #[route("/")]
    Home,
    #[route("/project/overview")]
    Overview,
    #[end_layout]
    #[route("/..route")]
    PageNotFound { },
}

#[allow(non_snake_case)]
//#[component]
fn Home() -> Element {
    rsx!(
        label {
            "Home"
        }
    )
}

#[allow(non_snake_case)]
//#[component]
fn Overview() -> Element {
    rsx!(
        label {
            "Overview"
        }
    )
}


#[allow(non_snake_case)]
//#[component]
fn PageNotFound() -> Element {
    rsx!(
        label {
            "Not Found"
        }
    )
}

#[allow(non_snake_case)]
fn AppSidebar() -> Element {

    let mut i18n = use_i18();
    
    let view = use_signal(ViewModel::default);

    let app_core = use_coroutine(|mut rx| {
        let svc = CoreService::new(view);
        async move { svc.run(&mut rx).await }
    });

    let on_click_create = move |_| {
        debug!("create clicked");
        app_core.send(planner_app::Event::CreateProject { project_name: "test".to_string(), path: Default::default() } );
    };

    let on_click_save = move |_| {
        debug!("save clicked");
        app_core.send(planner_app::Event::Save );
    };

    let selected_language = LANGUAGES.iter().find(|lang| {
        *i18n.selected_language.read() == lang.code
    }).unwrap();

    rsx!(
        NativeRouter {
            rect {
                width: "100%",
                direction: "horizontal",
                background: "#3C3F41",

                rect {
                    width: "60%",
                    direction: "horizontal",
                    Button {
                        onclick: on_click_create,
                        label {
                            {format!("\u{ea7b} {}", translate!(i18n, "messages.toolbar.button.create"))}
                        }
                    },
                    Button {
                        onclick: on_click_save,
                        label {
                            {format!("\u{e27c} {}", translate!(i18n, "messages.toolbar.button.save"))}
                        }
                    },
                },
                // TODO instead of specifying two rects, each with a width, have a spacer element here which takes
                //      up the remaining space so that the first rect is left justified and the second rect is right justified.
                //      and so that the window cannot be resized smaller than the width of the elements in the two rects.
                rect {
                    width: "40%",
                    direction: "horizontal",
                    main_align: "end",
                    // FIXME this additional rect is required because the dropdown inherits the direction from the parent
                    rect {
                       direction: "vertical",
                        Dropdown {
                            value: selected_language.name,
                            for language in LANGUAGES {
                                DropdownItem {
                                    value: language.code,
                                    onclick: {
                                        to_owned![language];
                                        move |_| i18n.set_language(LanguageIdentifier::from_str(language.code).unwrap())
                                    },
                                    label { "{language.name}" }
                                }
                            }
                        }
                    }
                }
            },

            Sidebar {
                sidebar: rsx!(
                    SidebarItem {
                        label {
                            "TODO"
                        }
                    },
                ),
                Body {
                    rect {
                        main_align: "center",
                        cross_align: "center",
                        width: "100%",
                        height: "100%",
                        Outlet::<Route> {  }
                    }
                }
            }
        }
    )
}

static ARIMO_NERD_FONT: &[u8] = include_bytes!("../assets/fonts/ArimoNerdFont-Regular.ttf");


static EN_US: &str = include_str!("../assets/i18n/en-US.json");
static ES_ES: &str = include_str!("../assets/i18n/es-ES.json");

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");
    console_error_panic_hook::set_once();
    
    launch_cfg(
        app,
        LaunchConfig::<()>::builder()
            .with_font("Arimo Nerd", ARIMO_NERD_FONT)
            .build(),
    );
}