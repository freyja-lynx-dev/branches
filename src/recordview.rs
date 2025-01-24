use std::fmt::Error;

use crate::types::AtUri;
use atrium_api::did_doc::*;
use atrium_api::types::string::{Cid, Did, Handle, Nsid};
use atrium_api::types::Unknown as AtUnknown;
use atrium_api::*;
use relm4::adw::prelude::*;
use relm4::factory::{AsyncFactoryComponent, FactoryView};
use relm4::gtk::prelude::*;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::{prelude::*, AsyncFactorySender};
use types::TryFromUnknown;

// #[derive(Debug)]
// pub struct GetRecordView {
//     uri: AtUri,
//     cid: Option<Cid>,
//     value: AtUnknown,
// }

// #[relm4::component(async)]
// impl SimpleAsyncComponent for GetRecordView {
//     view! {
//         gtk::ListBox {
//             set_hexpand: true,
//             set_margin_all: 10,
//         }
//     }
// }

#[derive(Debug)]
pub struct DescribeRepoView {
    collections: Vec<Nsid>,
    did: Did,
    did_doc: AtUnknown,
    handle: Handle,
    handle_is_correct: bool,
}
#[relm4::factory(async, pub)]
impl AsyncFactoryComponent for DescribeRepoView {
    type Init = com::atproto::repo::describe_repo::OutputData;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::TabView;
    view! {
        #[root]
        gtk::ListBox {
            set_hexpand: true,
            set_margin_all: 10,
            inline_css: "border-radius: 10px",
            #[name(identities)]
            adw::ExpanderRow {
                set_title: "Identities",
                set_expanded: false,
                add_row = &adw::ActionRow {
                    set_title: "DID",
                    set_subtitle: &format!("{}", &self.did.to_string()),
                    add_css_class: "property"
                },
                add_row = &adw::ActionRow {
                    set_title: "Handle",
                    set_subtitle: &format!("{}", &self.handle.to_string()),
                    add_css_class: "property"
                }
            },
            #[name(collections)]
            adw::ExpanderRow {
                set_title: "Collections",
                set_expanded: false,
            },
            #[name(did_doc)]
            adw::ExpanderRow {
                set_title: "DID Document",
                set_expanded: true,
            },
        },
        #[local_ref]
        returned_widget -> adw::TabPage {
            set_title: &format!("{}", &self.did.to_string())
        }
    }
    async fn init_model(
        value: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            collections: value.collections,
            did: value.did,
            did_doc: value.did_doc,
            handle: value.handle,
            handle_is_correct: value.handle_is_correct,
        }
    }
    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: AsyncFactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        for collection in &self.collections {
            let row = adw::ActionRow::new();
            row.set_title(&collection.to_string());
            widgets.collections.add_row(&row);
        }
        if let Ok(did_doc) = DidDocument::try_from_unknown(self.did_doc.clone()) {
            println!("did_doc verified: {:?}", did_doc);
            let context = adw::ExpanderRow::new();
            context.set_title("Context");
            match did_doc.context {
                Some(contexts) => {
                    for c in contexts {
                        let row = adw::ActionRow::new();
                        row.set_title(&c.to_string());
                        context.add_row(&row);
                    }
                }
                None => {
                    dbg!("no contexts in did_doc");
                    let row = adw::ActionRow::new();
                    row.set_title("No contexts in DID Document")
                }
            };
            widgets.did_doc.add_row(&context);

            let id = adw::ActionRow::new();
            id.set_title("id");
            id.set_subtitle(&did_doc.id.to_string());
            id.add_css_class("property");
            widgets.did_doc.add_row(&id);

            let also_known_as = adw::ExpanderRow::new();
            also_known_as.set_title("Also Known As");
            if let Some(aka_list) = did_doc.also_known_as {
                for also_known in aka_list {
                    let row = adw::ActionRow::new();
                    row.set_title(&also_known.to_string());
                    also_known_as.add_row(&row);
                }
            }
            widgets.did_doc.add_row(&also_known_as);

            let verification_methods = adw::ExpanderRow::new();
            verification_methods.set_title("Verification Methods");
            if let Some(vm_list) = did_doc.verification_method {
                for vm in vm_list {
                    relm4::view! {
                        vm_row = adw::ExpanderRow {
                            set_title: "Verification Method",
                            add_row = &adw::ActionRow {
                                set_title: "id",
                                set_subtitle: &vm.id,
                                add_css_class: "property"
                            },
                            add_row = &adw::ActionRow {
                                set_title: "type",
                                set_subtitle: &vm.r#type,
                                add_css_class: "property"
                            },
                            add_row = &adw::ActionRow {
                                set_title: "controller",
                                set_subtitle: &vm.controller.to_string(),
                                add_css_class: "property"
                            },
                        }
                    }
                    if let Some(public_key_multibase) = vm.public_key_multibase {
                        let public_key = adw::ActionRow::new();
                        public_key.set_title("publicKeyMultibase");
                        public_key.set_subtitle(&public_key_multibase);
                        public_key.add_css_class("property");
                        vm_row.add_row(&public_key);
                    }
                    verification_methods.add_row(&vm_row);
                }
            }
            widgets.did_doc.add_row(&verification_methods);

            let services = adw::ExpanderRow::new();
            services.set_title("Services");

            if let Some(service) = did_doc.service {
                for s in service {
                    relm4::view! {
                        service_row = adw::ExpanderRow {
                            set_title: "Service",
                            add_row = &adw::ActionRow {
                                set_title: "id",
                                set_subtitle: &s.id,
                                add_css_class: "property"
                            },
                            add_row = &adw::ActionRow {
                                set_title: "type",
                                set_subtitle: &s.r#type,
                                add_css_class: "property"
                            },
                            add_row = &adw::ActionRow {
                                set_title: "serviceEndpoint",
                                set_subtitle: &s.service_endpoint,
                                add_css_class: "property"
                            },
                        }
                    }
                    services.add_row(&service_row);
                }
                widgets.did_doc.add_row(&services);
            }
        } else {
            println!("invalid did_doc!");
        }
        widgets
    }
}

#[derive(Debug)]
pub struct ListRecordsView {
    cursor: Option<String>,
    records: Vec<com::atproto::repo::list_records::Record>,
}

// impl RecordView {
//     fn set_spinner(root: &<Self as Component>::Root, widget: &gtk::Widget) -> gtk::Widget {
//         root.remove(widget);
//         relm4::view! {
//             #[local_ref]
//             root -> gtk::Box {
//                 set_halign: gtk::Align::Center,
//                 set_valign: gtk::Align::Center,

//                 #[name(spinner)]
//                 gtk::Spinner {
//                     start: (),
//                     set_hexpand: true,
//                     set_vexpand: true,
//                 }
//             }
//         }
//         spinner.upcast()
//     }

//     fn set_record(
//         root: &<Self as Component>::Root,
//         widget: &gtk::Widget,
//         record: com::atproto::repo::get_record::OutputData,
//     ) -> gtk::Widget {
//         root.remove(widget);
//         relm4::view! {
//             #[local_ref]
//             root -> gtk::Box {
//                 set_halign: gtk::Align::Center,
//                 set_valign: gtk::Align::Center,

//                 #[local_ref]
//                 record ->
//         }
// }

// pub struct RecordView {
//     output: com::atproto::repo::get_record::Output,
//     parameters: com::atproto::repo::get_record::Parameters,
// }

// #[derive(Debug)]
// pub enum RecordViewMsg {
//     LoadRecord(String),
// }

// #[relm4::factory]
// impl FactoryComponent for RecordView {
//     type Init = String;
//     type Input = RecordViewMsg;
//     type Root =
//     type Output = ();
//     type CommandOutput = ();
//     type ParentWidget = adw::TabView;

//     view! {
//         gtk::Box {
//             set_orientation: gtk::Orientation::Vertical,
//             set_hexpand: true,
//             set_spacing: 12,

//             adw::Clamp {
//                 gtk::Frame{
//                     set_label: Some("placeholder")
//                 }
//             }
//         }
//     }
//     fn init(
//         str: Self::Init,
//         root: Self::Root,
//         sender: ComponentSender<Self>
//     ) -> ComponentParts<Self> {
//         let widget = gtk::
//     }
// }

// pub enum Record {
//     Record(com::atproto::repo::get_record::OutputData),
//     Records(com::atproto::repo::list_records::OutputData),
//     Repo(com::atproto::repo::describe_repo::OutputData),
// }

// pub struct RecordView {
//     data: Component,
// }

// pub enum RecordViewInput {
//     DisplayRecord(),
//     DisplayRecords(),
//     DisplayRepo(),
// }

// pub enum RecordViewOutput {
//     Get(AtUri),
// }
