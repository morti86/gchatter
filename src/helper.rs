use gtk::{DropDown, StringList};
use pv_recorder::PvRecorderBuilder;
use pulldown_cmark::{Parser, Options, html};
use regex::Regex;

// Gets the drop down with devices
pub fn device_dd() -> DropDown {
    let devices = PvRecorderBuilder::new(512)
        .get_available_devices()
        .unwrap_or_else(|e| {
            eprintln!("Cannot obtain the list of record devices!: {}", e.to_string());
            vec![]
        });

    let options = StringList::new(&[]);
    for d in devices {
        options.append(d.as_str());
    }

    DropDown::builder()
        .model(&options)
        .width_request(450)
        .margin_start(5)
        .build()
}

//--------------- Enums -------------

#[macro_export]
macro_rules! make_enum {
    ($name:ident, [$op1:ident, $($opt:ident),*]) => {
        #[derive(Clone, Debug, Copy, PartialEq)]
        pub enum $name {
            $op1,
            $(
                $opt,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$op1
            }
        }

        impl $name {
            // Fixed array with commas
            pub const ALL: &'static [Self] = &[$name::$op1, $($name::$opt),+];

            pub fn to_string(&self) -> String {
                match self {
                    $name::$op1 => stringify!($op1).to_string(),
                    $(
                        $name::$opt => stringify!($opt).to_string(),
                    )*
                }
            }

            pub fn as_str(&self) -> &str {
                match self {
                    $name::$op1 => stringify!($op1),
                    $(
                        $name::$opt => stringify!($opt),
                    )*
                }
            }
        }

        impl Into<$name> for String {
            fn into(self) -> $name {
                let s = self.as_str();
                match s {
                    stringify!($op1) => $name::$op1,
                    $(
                        stringify!($opt) => $name::$opt,
                    )*
                        _ => $name::$op1,
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(self.to_string().as_str())
            }
        }
    };
}

//-------- DropDown from enum -----------

#[macro_export]
macro_rules! enum_dd {
    ($name:ident) => {
        {
        let options = gtk::StringList::new(
            &$name::ALL
                .iter()
                .map(|a| a.as_str())
                .collect::<Vec<&str>>()
            );
        DropDown::builder()
            .model(&options)
            .build()
        }
    };
    ($name:ident, $m:expr) => {
        {
        let options = gtk::StringList::new(
            &$name::ALL
                .iter()
                .map(|a| a.as_str())
                .collect::<Vec<&str>>()
            );
        DropDown::builder()
            .model(&options)
            .margin_start($m)
            .build()
        }
    };
    ($name:ident, $m:expr, $w:expr) => {
        {
        let options = gtk::StringList::new(
            &$name::ALL
                .iter()
                .map(|a| a.as_str())
                .collect::<Vec<&str>>()
            );
        DropDown::builder()
            .model(&options)
            .width_request($w)
            .margin_start($m)
            .build()
        }
    };

}

//---------- DropDown from string array -------

#[macro_export]
macro_rules! dd {
    [$($a:expr),*] => {
        {
        let options = StringList::new(&[]);
        $(
            options.append($a);
        )*
        DropDown::builder()
            .model(&options)
            .build()
        }
    };
    ($m:expr, [$($a:expr),*]) => {
        {
        let options = StringList::new(&[]);
        $(
            options.append($a);
        )*
        DropDown::builder()
            .model(&options)
            .margin_start($m)
            .build()
        }
    };

}

//---------- Boxes ------------------

#[macro_export]
macro_rules! row {
    [$($a:ident),*] => {
        {
            let b = Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .build();
            $(
                b.append(&$a);
            )*

            b
        }
    };
    ($m:expr, [$($a:ident),*]) => {
        {
            let b = Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top($m)
                .margin_start($m)
                .build();
            $(
                b.append(&$a);
            )*

            b

        }
    };

}

#[macro_export]
macro_rules! column {
    [$($a:ident),*] => {
        {
            let b = Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();
            $(
                b.append(&$a);
            )*

            b
        }
    };
    ($m:expr, [$($a:ident),*]) => {
        {
            let b = Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .margin_top($m)
                .margin_start($m)
                .build();
            $(
                b.append(&$a);
            )*

            b
        }
    };

}

#[macro_export]
macro_rules! get_text {
    ($tb:ident) => {
        {
        let start_iter = $tb.start_iter();
        let end_iter = $tb.end_iter();
        $tb.text(&start_iter, &end_iter, false)
        }
    };

    ($tb:ident, $apps:ident) => {
        {
        let start_iter = $apps.text_buffer.start_iter();
        let end_iter = $apps.text_buffer.end_iter();
        $apps.$tb.text(&start_iter, &end_iter, false)
        }
    };
}

#[macro_export]
macro_rules! scrolled_text_view {
    () => {
        {
            let text_view = TextView::builder()
                .editable(true)
                .cursor_visible(true)
                .accepts_tab(true)
                .height_request(150)
                .wrap_mode(gtk::WrapMode::Word)
                .build();
            let scroll = ScrolledWindow::builder()
                .child(&text_view)
                .min_content_height(150)
                .build();
            scroll
        }
    };
    ($h:expr) => {
        {
            let text_view = TextView::builder()
                .editable(true)
                .cursor_visible(true)
                .accepts_tab(true)
                .height_request($h)
                .wrap_mode(gtk::WrapMode::Word)
                .build();
            let scroll = ScrolledWindow::builder()
                .child(&text_view)
                .min_content_height($h)
                .build();
            scroll
        }
    };
}

#[macro_export]
macro_rules! report_err {
    ($ex:expr) => {
        if let Err(e) = $ex {
            error!("{}", e.to_string());
        }
    }
}

#[macro_export]
macro_rules! clear_text {
    ($tb:ident) => {
        {
            let mut end_iter = $tb.end_iter();
            let mut star_iter = $tb.start_iter();
            $tb.delete(&mut star_iter, &mut end_iter);
        }
    }
}

pub fn convert_text(text: &str) -> String {

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(text, options);

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    
    let pango = html_to_pango(&html_out);
    pango
}

fn replace_tag(text: &str, from: &str, to: &str) -> String {
    let r_open = Regex::new(format!(r"<{}\b[^>]*>", from).as_str()).unwrap();
    let r_close = Regex::new(format!(r"</{}\b[^>]*>", from).as_str()).unwrap();

    let open = if to == "" { String::new() } else { format!("<{}>", to) };
    let close = if to == "" { String::new() } else { format!("</{}>", to) };
    let res = r_open.replace_all(text, open).to_string();
    let res = r_close.replace_all(res.as_str(), close).to_string();

    res
}

// Basic HTML to Pango markup converter (simplified)
fn html_to_pango(html: &str) -> String {
    let res = html.replace("<strong>", "<b>")
        .replace("</strong>", "</b>")
        .replace("<em>", "<i>")
        .replace("</em>", "</i>")
        .replace("<br>", "")
        .replace("<br/>", "")
        .replace("<li>", "- ")
        .replace("</li>", "")
        .replace("<pre>", "")
        .replace("</pre>", "")
        .replace("<h3>", "<span foreground=\"red\"><b>")
        .replace("</h3>", "</b></span>")
        .replace("<h2>", "<big><span foreground=\"green\">")
        .replace("</h2>", "</span></big>")
        .replace("<h1>", "<big><span foreground=\"red\">")
        .replace("</h1>", "</span></big>");

    let res = replace_tag(res.as_str(), "code", "tt");
    let res = replace_tag(res.as_str(), "ol", "");
    let res = replace_tag(res.as_str(), "ul", "");
    let res = replace_tag(res.as_str(), "p", "");

    let re_open = Regex::new(r"<h[4-6]>").unwrap();
    let res = re_open.replace_all(res.as_str(), "<big>").to_string();
    let re_open = Regex::new(r"<h/[4-6]>").unwrap();
    let res = re_open.replace_all(res.as_str(), "</big>").to_string();
    res.to_string()

        // Add more conversions as needed
}

