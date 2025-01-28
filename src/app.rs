use relm4::component::{
    AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncComponentSender,
    AsyncController,
};
use relm4::factory::AsyncFactoryVecDeque;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    adw,
    factory::FactoryVecDeque,
    gtk::{self, prelude::EntryBufferExtManual},
    main_application,
    prelude::DynamicIndex,
    Component, ComponentController, Controller, RelmWidgetExt,
};

use gtk::prelude::{
    ApplicationExt, ApplicationWindowExt, ButtonExt, EntryExt, GtkWindowExt, OrientableExt,
    SettingsExt, WidgetExt,
};
use gtk::{gio, glib};

use atrium_api::agent::{store::MemorySessionStore, AtpAgent};
use atrium_api::types::string::{AtIdentifier, Did, Handle};
use atrium_api::xrpc::Result as AtResult;
use atrium_api::*;

use crate::agent::{AgentInput, AgentOutput, AtprotoAgent};
use crate::config::{APP_ID, PROFILE};
use crate::modals::about::AboutDialog;
use crate::recordview::GetRecordView;
use crate::types::*;

pub(super) struct App {
    about_dialog: Controller<AboutDialog>,
    entry: gtk::EntryBuffer,
    views: AsyncFactoryVecDeque<GetRecordView>,
    created_widgets: u8,
    atp_client: AsyncController<AtprotoAgent>,
}

// #[derive(Debug)]
// pub(super) enum AppMsg {
//     Retrieve,
//     Quit,
// }

#[derive(Debug)]
pub enum AppMsg {
    DisplayOverview,
    // AddCounter,
    // RemoveCounter,
    // SendFront(DynamicIndex),
    // MoveUp(DynamicIndex),
    // MoveDown(DynamicIndex),
    Retrieve,
    TabForRecord(com::atproto::repo::get_record::OutputData),
    TabForRecords(com::atproto::repo::list_records::OutputData),
    TabForRepo(com::atproto::repo::describe_repo::OutputData),
    NotImplemented,
    Quit,
}

#[derive(Debug)]
pub enum AppCommand {
    GetRecord(
        AtResult<com::atproto::repo::get_record::Output, com::atproto::repo::get_record::Error>,
    ),
    ListRecords(
        AtResult<com::atproto::repo::list_records::Output, com::atproto::repo::list_records::Error>,
    ),
    DescribeRepo(
        AtResult<
            com::atproto::repo::describe_repo::Output,
            com::atproto::repo::describe_repo::Error,
        >,
    ),
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

#[relm4::component(pub, async)]
impl AsyncComponent for App {
    type CommandOutput = AppCommand;
    type Init = ();
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
                        #[wrap(Some)]
                        set_title_widget = &gtk::Box {
                            #[name(search_entry)]
                            gtk::Entry {
                                set_icon_from_icon_name: (gtk::EntryIconPosition::Primary, Some("edit-find-symbolic")),
                                set_width_request: 300,
                                set_hexpand: true,
                                set_tooltip_text: Some("Enter a handle, did, or at:// URI"),
                                set_buffer: &model.entry,
                                connect_activate => AppMsg::Retrieve,
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

    async fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let about_dialog = AboutDialog::builder()
            .transient_for(&root)
            .launch(())
            .detach();

        // let counters = FactoryVecDeque::builder()
        //     .launch(adw::TabView::default())
        //     .forward(sender.input_sender(), |output| match output {
        //         CounterOutput::SendFront(index) => AppMsg::SendFront(index),
        //         CounterOutput::MoveUp(index) => AppMsg::MoveUp(index),
        //         CounterOutput::MoveDown(index) => AppMsg::MoveDown(index),
        //     });
        let views = AsyncFactoryVecDeque::builder()
            .launch(adw::TabView::default())
            .forward(sender.input_sender(), |output| match output {
                _ => AppMsg::NotImplemented,
            });
        let model = Self {
            about_dialog,
            entry: gtk::EntryBuffer::default(),
            created_widgets: 0,
            views,
            atp_client: AtprotoAgent::builder().launch(()).forward(
                sender.input_sender(),
                |output| match output {
                    AgentOutput::Record(record) => {
                        println!("record: {:?}", record);
                        AppMsg::TabForRecord(record)
                    }
                    AgentOutput::Records(records) => {
                        println!("records: {:?}", records);
                        AppMsg::TabForRecords(records)
                    }
                    AgentOutput::Repo(repo) => {
                        println!("repo: {:?}", repo);
                        AppMsg::TabForRepo(repo)
                    }
                    AgentOutput::DidDoc(did_doc) => {
                        println!("did_doc: {:?}", did_doc);
                        AppMsg::NotImplemented
                    }
                    AgentOutput::PdsEndpoint(pds_endpoint) => {
                        println!("pds_endpoint: {:?}", pds_endpoint);
                        AppMsg::NotImplemented
                    }
                    AgentOutput::Error(err) => {
                        println!("error: {:?}", err);
                        AppMsg::NotImplemented
                    }
                },
            ),
        };

        let tab_view = model.views.widget();

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

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        let mut counters_guard = self.views.guard();

        match message {
            AppMsg::Retrieve => {
                if let Ok(uri) = self.entry.text().to_string().parse::<AtUri>() {
                    self.atp_client.emit(AgentInput::GetURI(uri));
                } else {
                    println!("not a valid URI"); // TODO create a toast to represent this error
                }
            }
            AppMsg::DisplayOverview => {
                widgets.tab_overview.set_open(true);
            }
            AppMsg::TabForRecord(record) => {
                counters_guard.push_back(record);
                self.created_widgets = self.created_widgets.wrapping_add(1);
            }
            AppMsg::TabForRecords(records) => {
                dbg!("we're not worrying about this for now");
                // counters_guard.push_back(records);
                // self.created_widgets = self.created_widgets.wrapping_add(1);
            }
            AppMsg::TabForRepo(repo) => {
                dbg!("we're not worrying about this for now");
                // counters_guard.push_back(repo);
                // self.created_widgets = self.created_widgets.wrapping_add(1);
            }
            AppMsg::NotImplemented => println!("not implemented"),
            AppMsg::Quit => main_application().quit(),
        }
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
