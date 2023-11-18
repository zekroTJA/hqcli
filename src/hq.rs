use std::{io::Write, sync::Arc};

use anyhow::Result;
use chrono::NaiveDateTime;
use headless_chrome::{browser::tab::ModifierKey, Browser, Tab};
use log::info;

pub struct HQ {
    _browser: Browser,
    tab: Arc<Tab>,
}

impl HQ {
    pub fn new(endpoint: &str, session: (&str, &str)) -> Result<Self> {
        info!("Initializing browser instance ...");

        let browser = Browser::default()?;
        let tab = browser.new_tab()?;

        let (session_key, session_secret) = session;

        info!("Initially load page ...");
        tab.navigate_to(endpoint)?;

        info!("Setting login session ...");
        tab.evaluate(
            &format!("localStorage.setItem(\"{session_key}\", \"{session_secret}\");"),
            false,
        )?;

        info!("Re-load page ...");
        tab.navigate_to(endpoint)?;

        tab.wait_for_element("div.primary-icon-add").map_err(|e| {
            anyhow::anyhow!(
                "Failed detecting 'Add' button: {e}\n\n\
                Maybe your credentials have been expired or you might not have \
                added the time log widget to your dashboard."
            )
        })?;

        Ok(HQ {
            _browser: browser,
            tab,
        })
    }

    pub fn log_worktime(&self, start: NaiveDateTime, end: NaiveDateTime) -> Result<()> {
        info!("Opening logging modal ...");
        self.tab.find_element("div.primary-icon-add")?.click()?;
        self.tab.wait_for_element("input[id^='datetimepicker']")?;

        let inputs = self.tab.find_elements("input[id^='datetimepicker']")?;
        if inputs.len() < 2 {
            anyhow::bail!("Could not find input elements in modal");
        }

        info!("Entering start date and time ...");
        inputs[0].click()?;
        self.tab
            .press_key_with_modifiers("A", Some(&[ModifierKey::Ctrl]))?;
        self.tab.type_str(&format_datetime(start))?;

        info!("Entering end date and time ...");
        inputs[1].click()?;
        self.tab
            .press_key_with_modifiers("A", Some(&[ModifierKey::Ctrl]))?;
        self.tab.type_str(&format_datetime(end))?;

        info!("Submitting form ...");
        self.tab.find_element("button#Save")?.click()?;

        info!("Successfully logged work time!");
        Ok(())
    }

    #[allow(dead_code)]
    fn take_debug_screenshot(&self) -> Result<()> {
        let d = self.tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;
        let mut f = std::fs::File::create("screenshot.png")?;
        f.write_all(d.as_slice())?;

        Ok(())
    }
}

fn format_datetime(dt: NaiveDateTime) -> String {
    format!("{}", dt.format("%d.%m.%y %H:%M"))
}
