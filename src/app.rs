use relm4::{
    actions::{RelmAction, RelmActionGroup},
    adw,
    factory::FactoryVecDeque,
    gtk, main_application,
    prelude::DynamicIndex,
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent,
};

use gtk::prelude::{
    ApplicationExt, ApplicationWindowExt, ButtonExt, EntryExt, GtkWindowExt, OrientableExt,
    SettingsExt, WidgetExt,
};
use gtk::{gio, glib};

use atrium_api::agent::{store::MemorySessionStore, AtpAgent};
use atrium_api::*;
use atrium_xrpc_client::reqwest::ReqwestClient;

use crate::config::{APP_ID, PROFILE};
use crate::modals::about::AboutDialog;
use crate::recordview::{Counter, CounterOutput};

pub(super) struct App {
    about_dialog: Controller<AboutDialog>,
    atp_agent: AtpAgent<MemorySessionStore, ReqwestClient>,
    counters: FactoryVecDeque<Counter>,
    created_widgets: u8,
}

// #[derive(Debug)]
// pub(super) enum AppMsg {
//     Retrieve,
//     Quit,
// }

#[derive(Debug)]
pub enum AppMsg {
    DisplayOverview,
    AddCounter,
    RemoveCounter,
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
    Quit,
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    menu! {
        primary_menu: {
            section! {
                "_Preferences" => PreferencesAction,
                "_Keyboard" => ShortcutsAction,
                "_About Branches" => AboutAction,
            }
        }
    }

    view! {
        main_window = adw::ApplicationWindow::new(&main_application()) {
            set_visible: true,

            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::Quit);
                glib::Propagation::Stop
            },

            #[wrap(Some)]
            set_help_overlay: shortcuts = &gtk::Builder::from_resource(
                    "/dev/freyja_lynx/Branches/gtk/help-overlay.ui"
                )
                .object::<gtk::ShortcutsWindow>("help_overlay")
                .unwrap() -> gtk::ShortcutsWindow {
                    set_transient_for: Some(&main_window),
                    set_application: Some(&main_application()),
            },

            add_css_class?: if PROFILE == "Devel" {
                    Some("devel")
                } else {
                    None
                },
            #[name = "tab_overview"]
            adw::TabOverview {
                set_enable_new_tab: true,
                set_view: Some(tab_view),
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::HeaderBar {
                        pack_start = &gtk::Button {
                            set_label: "Add",
                            connect_clicked => AppMsg::AddCounter,
                        },

                        pack_start = &gtk::Button {
                            set_label: "Remove",
                            connect_clicked => AppMsg::RemoveCounter,
                        },
                        #[wrap(Some)]
                        set_title_widget = &gtk::Box {
                            gtk::Entry {
                                set_icon_from_icon_name: (gtk::EntryIconPosition::Primary, Some("edit-find-symbolic")),
                                set_width_request: 300,
                                set_hexpand: true,
                                // connect_activate[sender] => move |_| {
                                //     sender.input(AppMsg::Retrieve)
                                // }
                            },
                            adw::TabButton {
                                set_view: Some(tab_view),
                                connect_clicked => AppMsg::DisplayOverview,
                            }
                        },
                        pack_end = &gtk::MenuButton {
                            set_icon_name: "open-menu-symbolic",
                            set_menu_model: Some(&primary_menu),
                        }
                    },
                    // adw::TabBar {
                    //     set_view: Some(tab_view),
                    //     set_autohide: false,
                    // },
                    #[local_ref]
                    tab_view -> adw::TabView {
                        set_margin_all: 5,
                        set_vexpand: true,
                    },
                    gtk::Label {
                        set_label: "Browse public AT Protocol data",
                    }
                }

            }

        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let about_dialog = AboutDialog::builder()
            .transient_for(&root)
            .launch(())
            .detach();

        let atp_agent: AtpAgent<MemorySessionStore, _> = AtpAgent::new(
            ReqwestClient::new("https://bsky.social"),
            MemorySessionStore::default(),
        );

        let counters = FactoryVecDeque::builder()
            .launch(adw::TabView::default())
            .forward(sender.input_sender(), |output| match output {
                CounterOutput::SendFront(index) => AppMsg::SendFront(index),
                CounterOutput::MoveUp(index) => AppMsg::MoveUp(index),
                CounterOutput::MoveDown(index) => AppMsg::MoveDown(index),
            });
        let model = Self {
            about_dialog,
            atp_agent,
            created_widgets: counter,
            counters,
        };

        let tab_view = model.counters.widget();

        let widgets = view_output!();

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();

        let shortcuts_action = {
            let shortcuts = widgets.shortcuts.clone();
            RelmAction::<ShortcutsAction>::new_stateless(move |_| {
                shortcuts.present();
            })
        };

        let about_action = {
            let sender = model.about_dialog.sender().clone();
            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.send(()).unwrap();
            })
        };

        actions.add_action(shortcuts_action);
        actions.add_action(about_action);
        actions.register_for_widget(&widgets.main_window);

        widgets.load_window_size();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        let mut counters_guard = self.counters.guard();

        match message {
            AppMsg::DisplayOverview => {
                println!("not implemented")
                //self.tab_overview.set_open(true);
            }
            AppMsg::AddCounter => {
                counters_guard.push_back(self.created_widgets);
                self.created_widgets = self.created_widgets.wrapping_add(1);
            }
            AppMsg::RemoveCounter => {
                counters_guard.pop_back();
            }
            AppMsg::SendFront(index) => {
                counters_guard.move_front(index.current_index());
            }
            AppMsg::MoveDown(index) => {
                let index = index.current_index();
                let new_index = index + 1;
                // Already at the end?
                if new_index < counters_guard.len() {
                    counters_guard.move_to(index, new_index);
                }
            }
            AppMsg::MoveUp(index) => {
                let index = index.current_index();
                // Already at the start?
                if index != 0 {
                    counters_guard.move_to(index, index - 1);
                }
            }
            AppMsg::Quit => main_application().quit(),
        }
        // match message {
        //     AppMsg::Retrieve => {
        //         println!("not implemented")
        //         // let res = match self
        //         //     .atp_agent
        //         //     .api
        //         //     .com
        //         //     .atproto
        //         //     .identity
        //         //     .resolve_handle(
        //         //         atrium_api::com::atproto::identity::resolve_handle::ParametersData {
        //         //             handle: atrium_api::types::string::Handle::from("freyja-lynx.dev"),
        //         //         }
        //         //         .into(),
        //         //     )
        //         //     .await
        //         // {
        //         //     Ok(r) => r.data.did,
        //         //     Err(e) => "not found",
        //         // };
        //         // println!("{:?}", res)
        //     }
        //     AppMsg::Quit => main_application().quit(),
        // }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.save_window_size().unwrap();
    }
}

impl AppWidgets {
    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = gio::Settings::new(APP_ID);
        let (width, height) = self.main_window.default_size();

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;

        settings.set_boolean("is-maximized", self.main_window.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = gio::Settings::new(APP_ID);

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.main_window.set_default_size(width, height);

        if is_maximized {
            self.main_window.maximize();
        }
    }
}
