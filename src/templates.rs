use gtk::prelude::*;
use relm4::{gtk, RelmWidgetExt, WidgetTemplate};

#[relm4::widget_template(pub)]
impl WidgetTemplate for KeyLabel {
    view! {
        gtk::Label {
            inline_css: "font-weight: bold; font-family: monospace, monospace"
        },
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for SimpleKeyValue {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,
            set_margin_all: 10,

            #[name(key)]
            #[template]
            KeyLabel,
            #[name(value)]
            gtk::Label {
                inline_css: "font-family: monospace, monospace"
            }
        }
    }
}
#[relm4::widget_template(pub)]
impl WidgetTemplate for NestedKeyValue {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 10,
            #[name(key)]
            #[template]
            KeyLabel,
            #[name(value)]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
            },
        }
    }
}
#[relm4::widget_template(pub)]
impl WidgetTemplate for AppBskyFeedPost {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 10,

            #[name(text)]
            #[template]
            SimpleKeyValue,
            #[name(r#type)]
            #[template]
            SimpleKeyValue,
            #[name(embed)]
            #[template]
            NestedKeyValue {},
            #[name(langs)]
            #[template]
            NestedKeyValue {},
            #[name(created_at)]
            #[template]
            SimpleKeyValue,
        }
    }
}
