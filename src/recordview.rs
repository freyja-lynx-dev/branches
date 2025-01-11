// use atrium_api::*;
use relm4::adw::prelude::*;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

// pub struct RecordView {}

// #[relm4::factory]
// impl FactoryComponent for RecordView {
//     type ParentWidget = adw::TabView;
// }
#[derive(Debug)]
pub struct Counter {
    value: u8,
}

#[derive(Debug)]
pub enum CounterMsg {
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum CounterOutput {
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for Counter {
    type Init = u8;
    type Input = CounterMsg;
    type Output = CounterOutput;
    type CommandOutput = ();
    type ParentWidget = adw::TabView;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 12,

            gtk::Box {
                set_spacing: 12,

                #[name(label)]
                gtk::Label {
                    #[watch]
                    set_label: &self.value.to_string(),
                    set_width_chars: 3,
                },

                gtk::Button {
                    set_label: "+",
                    connect_clicked => CounterMsg::Increment,
                },

                gtk::Button {
                    set_label: "-",
                    connect_clicked => CounterMsg::Decrement,
                },

                gtk::Button {
                    set_label: "Up",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(CounterOutput::MoveUp(index.clone())).unwrap()
                    }
                },

                gtk::Button {
                    set_label: "Down",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(CounterOutput::MoveDown(index.clone())).unwrap()
                    }
                },

                gtk::Button {
                    set_label: "To Start",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(CounterOutput::SendFront(index.clone())).unwrap()
                    }
                }
            }
        },
        #[local_ref]
        returned_widget -> adw::TabPage {
            set_title: &format!("Page {}", self.value),
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { value }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            CounterMsg::Increment => {
                self.value = self.value.wrapping_add(1);
            }
            CounterMsg::Decrement => {
                self.value = self.value.wrapping_sub(1);
            }
        }
    }
}
