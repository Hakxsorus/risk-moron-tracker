use crate::detector::{ScanInfo};
use crate::{detector, paths};
use iced::font::Style;
use iced::font::Weight::{Bold};
use iced::widget::image::Handle;
use iced::widget::{
    self, container, text, Column, Row
};
use iced::{
    color, Alignment, Element, Length, Padding, Sandbox, Theme
};

pub(crate) struct BlitzApp {
    error: Option<String>,
    scans: Vec<ScanInfo>,
    done_initial_scan: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum BlitzMessage {
    OpenBlacklistFIle,
    OpenSupportUrl,
    ScanRisk
}

impl Sandbox for BlitzApp {
    type Message = BlitzMessage;

    fn new() -> Self {
        Self {
            error: None,
            scans: Vec::new(),
            done_initial_scan: false,
        }
    }

    fn title(&self) -> String {
        String::from("Blitz - The RISK Moron Detector")
    }

    fn update(&mut self, message: BlitzMessage) {
        match message {
            // Open the blacklist file in the default text editor.
            BlitzMessage::OpenBlacklistFIle => {
                match paths::blacklist_path() {
                    Some(path) => open::that(&path).unwrap_or_else(|err| {
                        self.error = Some(err.to_string());
                    }),
                    None => {
                        self.error = Some(String::from("Unable to find the path to the blacklist."))
                    }
                }
            },
            // Open the support URL in the default browser.
            BlitzMessage::OpenSupportUrl => {
                open::that(paths::SUPPORT_URL).unwrap_or_else(|err| {
                    self.error = Some(err.to_string());
                })
            },
            // Scan the RISK application for morons.
            BlitzMessage::ScanRisk => {
                self.error = Some(String::from("Scanning - Please wait."));
                match detector::scan() {
                    Ok(scans) => {
                        self.scans = scans;
                        self.done_initial_scan = true;
                        self.error = None;
                    }
                    Err(err) => {
                        self.error = Some(String::from(err.to_string()));
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<BlitzMessage> {
        let banner_row_maybe = create_banner_row();
        let button_row = create_button_row();
        let scan_row = create_scan_row(self.done_initial_scan, &self.scans);
        let error_row = create_error_row(self.error.as_deref());

        // Push the master column with all the UI elements into the container and publish.
        let mut master_column = Column::new().align_items(Alignment::Center);

        if let Some(banner_row) = banner_row_maybe {
            master_column = master_column.push(banner_row);
        };

        master_column = master_column
        .push(button_row)
        .push(scan_row)
        .push(error_row);

        container(master_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::KanagawaDragon
    }
}

impl BlitzApp {

}

/// Creates the banner [`Row`] for the application view. If the banner path cannot be constructed,
/// this function returns [`None`].
fn create_banner_row() -> Option<Element<'static, BlitzMessage>> {
    let banner_path = match paths::banner_path() {
        Some(banner_path) => banner_path,
        None => {
            // It's not the end of the world if we can't retrieve this.
            eprintln!("Unable to construct the banner path.");
            return None;
        }
    };

    let banner_image = widget::Image::new(Handle::from_path(&banner_path))
        .width(Length::Shrink)
        .height(Length::Shrink);
    let banner_row = Row::new()
        .align_items(Alignment::Center)
        .padding(pad(18, 14, 14, 0))
        .push(banner_image)
        .into();

    Some(banner_row)
}

/// Creates the button [`Row`] for the application view that contains the blacklist,
/// scan, and support buttons.
fn create_button_row() -> Element<'static, BlitzMessage> {
    let blacklist_button = widget::Button::new("Blacklist")
        .on_press(BlitzMessage::OpenBlacklistFIle);
    let scan_button = widget::Button::new("Scan")
        .on_press(BlitzMessage::ScanRisk);
    let support_button = widget::Button::new("Support")
        .on_press(BlitzMessage::OpenSupportUrl);

    widget::Row::new()
        .align_items(Alignment::Center)
        .spacing(10)
        .padding(pad(6, 14, 14, 0))
        .push(blacklist_button)
        .push(scan_button)
        .push(support_button)
        .into()
}

/// Creates the scan [`Row`] for the application view that contains the list of 
/// scanned morons, a message that says no morons were found, or a prompt to scan.
fn create_scan_row(done_initial_scan: bool, scans: &Vec<ScanInfo> ) -> Element<'static, BlitzMessage> {
    let mut scan_row = Row::new()
        .align_items(Alignment::Start)
        .padding(pad(10, 14, 14, 0));

    if done_initial_scan == false {
        scan_row = scan_row.push(text("Press SCAN to start detecting morons.").shaping(text::Shaping::Advanced));
        return scan_row.into()
    }

    let mut similar_scans: Vec<_> = scans
        .iter()
        .filter(|s| s.similarity >= 70)
        .collect();

    if similar_scans.is_empty() {
        scan_row = scan_row.push(text("No Morons Here (✿◠‿◠)").shaping(text::Shaping::Advanced));
        return scan_row.into()
    }

    // Sort the scans in descending order and push them into their individual columns.
    similar_scans.sort_by(|a, b| b.similarity.cmp(&a.similarity));

    let mut warning_column = widget::Column::new()
        .align_items(Alignment::Start)
        .padding(5);

    let mut username_column = widget::Column::new()
        .align_items(Alignment::Start)
        .padding(5);

    let mut similarity_column = widget::Column::new()
        .align_items(Alignment::Start)
        .padding(5);

    for similar_scan in similar_scans {
        warning_column = warning_column.push(text("MORON?").style(red()).font(bold()));
        username_column = username_column.push(text(&similar_scan.username).style(silver()));
        similarity_column = similarity_column.push(text(format!("({}%)", &similar_scan.similarity)).font(italic()))
    }

    scan_row
        .push(warning_column)
        .push(username_column)
        .push(similarity_column)
        .into()
}

/// Creates the button [`Row`] for the application view that contains the blacklist,
/// scan, and support buttons.
fn create_error_row(error: Option<&str>) -> Element<'static, BlitzMessage> {
    let message = match error {
        Some(error) => error,
        None => ""
    };

    widget::Row::new()
        .align_items(Alignment::Center)
        .push(text(message).style(red()))
        .into()
}


/// Constructs a new [`iced::Padding`] with the specified padding values.
///
/// # Arguments
/// * `top` - The value for the top edge of the padding.
/// * `left` - The value for the left edge of the padding.
/// * `right` - The value for the right edge of the padding.
/// * `bottom` - The value for the bottom edge of the padding.
fn pad(top: u32, left: u32, right: u32, bottom: u32) -> iced::Padding {
    Padding {
        top: top as f32,
        left: left as f32,
        right: right as f32,
        bottom: bottom as f32,
    }
}

/// Constructs a red [`iced::Color`].
fn red() -> iced::Color {
    color!(255, 0, 0)
}

/// Constructs a grey [`iced::Color`].
fn silver() -> iced::Color {
    color!(237, 237, 237)
}

/// Creates a bold [`iced::Font`].
fn bold() -> iced::Font {
    iced::Font {
        weight: Bold, 
        ..Default::default()
    }
}

/// Creates a bold and italicised [`iced::Font``].
fn italic() -> iced::Font {
    iced::Font {
        style: Style::Italic,
        ..Default::default()
    }
}