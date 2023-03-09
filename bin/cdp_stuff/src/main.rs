use cdp::socket::ChromiumBrowser;
use cdp::Capabilities;

#[tokio::main]
async fn main() {
    let mut caps = Capabilities::new();
    caps.set_binary("/home/mira/Projects/misc/chromium/src/out/Default/chrome");

    let args: Vec<String> = [
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
    ].map(String::from).to_vec();
    // caps.args = args;

    // DevTools listening on ws://127.0.0.1:34327/devtools/browser/ab89d15b-53ac-4f64-91ce-630661572e91
    let driver = ChromiumBrowser::new(caps).await;

    // foo();
    println!("hello world")
}
