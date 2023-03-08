mod tab_evasions;

use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::Result;
use headless_chrome::{
    protocol::cdp::Page::CaptureScreenshotFormatOption, types::Bounds, Browser, LaunchOptions,
};
use tab_evasions::TabEvasions;

fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

// this passes.
// https://hmaker.github.io/selenium-detector/
fn solve_selenium_detector(browser: &Browser) -> Result<()> {
    let tab = browser.new_tab()?;

    // tab.call_method(method)
    tab.navigate_to("https://hmaker.github.io/selenium-detector/")?
        .wait_for_element("#chromedriver-test-input")?;
    let token_v = tab
        .evaluate("window.token", false)?
        .value
        .unwrap()
        .to_string();

    let token = tab.find_element("#chromedriver-token").unwrap();
    // token.type_into(&token_v).unwrap();
    token.click().unwrap();
    tab.type_str(rem_first_and_last(&token_v)).unwrap();

    let token_v: String = tab
        .evaluate("window.getAsyncToken()", true)?
        .value
        .unwrap()
        .to_string();

    let token = tab.find_element("#chromedriver-asynctoken").unwrap();
    token.click().unwrap();
    tab.type_str(rem_first_and_last(&token_v)).unwrap();

    let submit = tab.find_element("#chromedriver-test").unwrap();
    submit.click().unwrap();

    thread::sleep(Duration::from_millis(2000));

    // screenshot
    let png_data =
        tab.capture_screenshot(CaptureScreenshotFormatOption::Png, Some(75), None, false)?;

    fs::write("selenium_detector.png", png_data)?;

    Ok(())
}

// https://hmaker.github.io/selenium-detector/
fn launch_creepjs(browser: &Browser) -> Result<()> {
    let tab = browser.new_tab()?;

    // todo: evasions, fingerprints
    TabEvasions::bypass_webgl_vendor(&tab).unwrap();

    tab.navigate_to("https://abrahamjuliot.github.io/creepjs/")?.wait_for_element(".visitor-info > div:nth-child(2) > div:nth-child(2) > div:nth-child(2) > span.unblurred")?;

    // set view bounds for full-page screenshot
    let body_height = tab.find_element("body")?.get_box_model()?.height;
    let body_width = tab.find_element("body")?.get_box_model()?.width;
    tab.set_bounds(Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width: Some(body_width),
        height: Some(body_height),
    })?;

    thread::sleep(Duration::from_millis(500));
    let png_data =
        tab.capture_screenshot(CaptureScreenshotFormatOption::Png, Some(90), None, true)?;

    fs::write("creepjs.png", png_data)?;

    // reset bounds
    tab.set_bounds(Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width: Some(1920.0),
        height: Some(1080.0),
    })?;

    Ok(())
}

fn launch_datadome(browser: &Browser) -> Result<()> {
    let tab = browser.new_tab()?;

    let png_data = tab
        .navigate_to("https://antoinevastel.com/bots/datadome")?
        .wait_until_navigated()?
        .capture_screenshot(CaptureScreenshotFormatOption::Png, Some(75), None, false)?;

    fs::write("datadome.png", png_data)?;

    Ok(())
}

fn main() -> Result<()> {
    // let xs = vec!["--disable-blink-features=AutomationControlled", "--"];

    let args = vec![
        "--disable-background-networking",
        "--enable-features=NetworkService,NetworkServiceInProcess",
        "--disable-background-timer-throttling",
        "--disable-backgrounding-occluded-windows",
        "--disable-breakpad",
        "--disable-client-side-phishing-detection",
        "--disable-component-extensions-with-background-pages",
        "--disable-component-update",
        "--disable-default-apps",
        "--disable-dev-shm-usage",
        "--disable-extensions",
        // BlinkGenPropertyTrees disabled due to crbug.com/937609
        "--disable-features=TranslateUI,BlinkGenPropertyTrees,AudioServiceOutOfProcess,IsolateOrigins,site-per-process",
        "--disable-blink-features=AutomationControlled",
        "--disable-hang-monitor",
        "--disable-ipc-flooding-protection",
        "--disable-popup-blocking",
        "--disable-prompt-on-repost",
        "--disable-renderer-backgrounding",
        "--disable-sync",
        "--disable-tab-for-desktop-share",
        "--disable-infobars",
        "--force-color-profile=srgb",
        "--metrics-recording-only",
        "--no-first-run",
        "--no-default-browser-check",
        "--ignore-gpu-blocklist",
        "--password-store=basic",
        "--use-mock-keychain",
        "--start-maximized",
        // headless=new also passes datadome
        "--headless=new",
        "--remote-debugging-port=9222",
        "--enable-webgl",
    ];

    let chrome_binary: PathBuf =
        Path::new("/home/mira/Projects/misc/chromium/src/out/Default/chrome").into();
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(false)
            .path(Some(chrome_binary))
            .disable_default_args(true)
            .args(args.iter().map(OsStr::new).collect())
            .window_size(Some((1920, 1080)))
            // .args
            .build()
            .expect("Could not find chrome-executable"),
    )?;

    launch_creepjs(&browser).unwrap();
    solve_selenium_detector(&browser).unwrap();
    launch_datadome(&browser).unwrap();

    io::stdin().read_line(&mut String::new()).unwrap();
    Ok(())
}
