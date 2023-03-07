use std::path::Path;

use serde_json::json;
use thirtyfour::{extensions::cdp::ChromeDevTools, prelude::*};
use tokio::time::{sleep, Duration};

async fn configure_headless(driver: &WebDriver) {
    let is_webdriver = driver
        .execute("return navigator.webdriver", Vec::new())
        .await
        .unwrap()
        .convert::<bool>()
        .unwrap();

    println!("navigator.webdriver: {}", is_webdriver);

    let dev_tools = ChromeDevTools::new(driver.handle.clone());

    println!("patching user-agent string");
    let new_user_agent = driver
        .execute("return navigator.userAgent", Vec::new())
        .await
        .unwrap()
        .convert::<String>()
        .unwrap()
        .replace("Headless", "");

    dev_tools
        .execute_cdp_with_params(
            "Network.setUserAgentOverride",
            json!({ "userAgent": new_user_agent }),
        )
        .await
        .unwrap();

    println!("adding chrome evasions");
    dev_tools
    .execute_cdp_with_params(
        "Page.addScriptToEvaluateOnNewDocument",
        json!({
            "source": r#"
            // navigator props
            Object.defineProperty(navigator, "maxTouchPoints", { get: () => 1 });
            Object.defineProperty(navigator.connection, "rtt", { get: () => 100 });

            // https://github.com/microlinkhq/browserless/blob/master/packages/goto/src/evasions/window-frame.js
            if (window.outerWidth && window.outerHeight) return;
            const windowFrame = 85;
            window.outerWidth = window.innerWidth;
            window.outerHeight = window.innerHeight + windowFrame;
            
            // https://github.com/microlinkhq/browserless/blob/master/packages/goto/src/evasions/chrome-runtime.js
            window.chrome = {
              app: {
                isInstalled: false,
                InstallState: {
                  DISABLED: "disabled",
                  INSTALLED: "installed",
                  NOT_INSTALLED: "not_installed",
                },
                RunningState: {
                  CANNOT_RUN: "cannot_run",
                  READY_TO_RUN: "ready_to_run",
                  RUNNING: "running",
                },
              },
              runtime: {
                OnInstalledReason: {
                  CHROME_UPDATE: "chrome_update",
                  INSTALL: "install",
                  SHARED_MODULE_UPDATE: "shared_module_update",
                  UPDATE: "update",
                },
                OnRestartRequiredReason: {
                  APP_UPDATE: "app_update",
                  OS_UPDATE: "os_update",
                  PERIODIC: "periodic",
                },
                PlatformArch: {
                  ARM: "arm",
                  ARM64: "arm64",
                  MIPS: "mips",
                  MIPS64: "mips64",
                  X86_32: "x86-32",
                  X86_64: "x86-64",
                },
                PlatformNaclArch: {
                  ARM: "arm",
                  MIPS: "mips",
                  MIPS64: "mips64",
                  X86_32: "x86-32",
                  X86_64: "x86-64",
                },
                PlatformOs: {
                  ANDROID: "android",
                  CROS: "cros",
                  LINUX: "linux",
                  MAC: "mac",
                  OPENBSD: "openbsd",
                  WIN: "win",
                },
                RequestUpdateCheckStatus: {
                  NO_UPDATE: "no_update",
                  THROTTLED: "throttled",
                  UPDATE_AVAILABLE: "update_available",
                },
              },
            };
            // https://github.com/microlinkhq/browserless/blob/master/packages/goto/src/evasions/navigator-permissions.js
            if (!window.Notification) {
              window.Notification = {
                permission: "denied",
              };
            }
            const originalQuery = window.navigator.permissions.query;
            window.navigator.permissions.__proto__.query = (parameters) =>
              parameters.name === "notifications"
                ? Promise.resolve({ state: window.Notification.permission })
                : originalQuery(parameters);
            const oldCall = Function.prototype.call;
            function call() {
              return oldCall.apply(this, arguments);
            }
            Function.prototype.call = call;
            const nativeToStringFunctionString = Error.toString().replace(
              /Error/g,
              "toString"
            );
            const oldToString = Function.prototype.toString;
            function functionToString() {
              if (this === window.navigator.permissions.query) {
                return "function query() { [native code] }";
              }
              if (this === functionToString) {
                return nativeToStringFunctionString;
              }
              return oldCall.call(oldToString, this);
            }
            // eslint-disable-next-line
            Function.prototype.toString = functionToString;
            // https://github.com/microlinkhq/browserless/blob/master/packages/goto/src/evasions/navigator-plugins.js
            function mockPluginsAndMimeTypes() {
              /* global MimeType MimeTypeArray PluginArray */
            
              // Disguise custom functions as being native
              const makeFnsNative = (fns = []) => {
                const oldCall = Function.prototype.call;
                function call() {
                  return oldCall.apply(this, arguments);
                }
                // eslint-disable-next-line
                Function.prototype.call = call;
            
                const nativeToStringFunctionString = Error.toString().replace(
                  /Error/g,
                  "toString"
                );
                const oldToString = Function.prototype.toString;
            
                function functionToString() {
                  for (const fn of fns) {
                    if (this === fn.ref) {
                      return `function ${fn.name}() { [native code] }`;
                    }
                  }
            
                  if (this === functionToString) {
                    return nativeToStringFunctionString;
                  }
                  return oldCall.call(oldToString, this);
                }
                // eslint-disable-next-line
                Function.prototype.toString = functionToString;
              };
            
              const mockedFns = [];
            
              const fakeData = {
                mimeTypes: [
                  {
                    type: "application/pdf",
                    suffixes: "pdf",
                    description: "",
                    __pluginName: "Chrome PDF Viewer",
                  },
                  {
                    type: "application/x-google-chrome-pdf",
                    suffixes: "pdf",
                    description: "Portable Document Format",
                    __pluginName: "Chrome PDF Plugin",
                  },
                  {
                    type: "application/x-nacl",
                    suffixes: "",
                    description: "Native Client Executable",
                    // eslint-disable-next-line
                    enabledPlugin: Plugin,
                    __pluginName: "Native Client",
                  },
                  {
                    type: "application/x-pnacl",
                    suffixes: "",
                    description: "Portable Native Client Executable",
                    __pluginName: "Native Client",
                  },
                ],
                plugins: [
                  {
                    name: "Chrome PDF Plugin",
                    filename: "internal-pdf-viewer",
                    description: "Portable Document Format",
                  },
                  {
                    name: "Chrome PDF Viewer",
                    filename: "mhjfbmdgcfjbbpaeojofohoefgiehjai",
                    description: "",
                  },
                  {
                    name: "Native Client",
                    filename: "internal-nacl-plugin",
                    description: "",
                  },
                ],
                fns: {
                  namedItem: (instanceName) => {
                    // Returns the Plugin/MimeType with the specified name.
                    const fn = function (name) {
                      if (!arguments.length) {
                        throw new TypeError(
                          `Failed to execute 'namedItem' on '${instanceName}': 1 argument required, but only 0 present.`
                        );
                      }
                      return this[name] || null;
                    };
                    mockedFns.push({ ref: fn, name: "namedItem" });
                    return fn;
                  },
                  item: (instanceName) => {
                    // Returns the Plugin/MimeType at the specified index into the array.
                    const fn = function (index) {
                      if (!arguments.length) {
                        throw new TypeError(
                          `Failed to execute 'namedItem' on '${instanceName}': 1 argument required, but only 0 present.`
                        );
                      }
                      return this[index] || null;
                    };
                    mockedFns.push({ ref: fn, name: "item" });
                    return fn;
                  },
                  refresh: (instanceName) => {
                    // Refreshes all plugins on the current page, optionally reloading documents.
                    const fn = function () {
                      return undefined;
                    };
                    mockedFns.push({ ref: fn, name: "refresh" });
                    return fn;
                  },
                },
              };
              // Poor mans _.pluck
              const getSubset = (keys, obj) =>
                keys.reduce((a, c) => ({ ...a, [c]: obj[c] }), {});
            
              function generateMimeTypeArray() {
                const arr = fakeData.mimeTypes
                  .map((obj) => getSubset(["type", "suffixes", "description"], obj))
                  .map((obj) => Object.setPrototypeOf(obj, MimeType.prototype));
                arr.forEach((obj) => {
                  arr[obj.type] = obj;
                });
            
                // Mock functions
                arr.namedItem = fakeData.fns.namedItem("MimeTypeArray");
                arr.item = fakeData.fns.item("MimeTypeArray");
            
                return Object.setPrototypeOf(arr, MimeTypeArray.prototype);
              }
            
              const mimeTypeArray = generateMimeTypeArray();
              Object.defineProperty(navigator, "mimeTypes", {
                get: () => mimeTypeArray,
              });
            
              function generatePluginArray() {
                const arr = fakeData.plugins
                  .map((obj) => getSubset(["name", "filename", "description"], obj))
                  .map((obj) => {
                    const mimes = fakeData.mimeTypes.filter(
                      (m) => m.__pluginName === obj.name
                    );
                    // Add mimetypes
                    mimes.forEach((mime, index) => {
                      navigator.mimeTypes[mime.type].enabledPlugin = obj;
                      obj[mime.type] = navigator.mimeTypes[mime.type];
                      obj[index] = navigator.mimeTypes[mime.type];
                    });
                    obj.length = mimes.length;
                    return obj;
                  })
                  .map((obj) => {
                    // Mock functions
                    obj.namedItem = fakeData.fns.namedItem("Plugin");
                    obj.item = fakeData.fns.item("Plugin");
                    return obj;
                  })
                  // eslint-disable-next-line
                  .map((obj) => Object.setPrototypeOf(obj, Plugin.prototype));
                arr.forEach((obj) => {
                  arr[obj.name] = obj;
                });
            
                // Mock functions
                arr.namedItem = fakeData.fns.namedItem("PluginArray");
                arr.item = fakeData.fns.item("PluginArray");
                arr.refresh = fakeData.fns.refresh("PluginArray");
            
                return Object.setPrototypeOf(arr, PluginArray.prototype);
              }
            
              const pluginArray = generatePluginArray();
              Object.defineProperty(navigator, "plugins", {
                get: () => pluginArray,
              });
            
              // Make mockedFns toString() representation resemble a native function
              makeFnsNative(mockedFns);
            }
            try {
              const isPluginArray = navigator.plugins instanceof PluginArray;
              const hasPlugins = isPluginArray && navigator.plugins.length > 0;
              if (isPluginArray && hasPlugins) {
                return; // nothing to do here
              }
              mockPluginsAndMimeTypes();
            } catch (err) {}
            // https://github.com/microlinkhq/browserless/blob/master/packages/goto/src/evasions/webgl-vendor.js
            // Remove traces of our Proxy ;-)
            const stripErrorStack = (stack) =>
              stack
                .split("\n")
                .filter((line) => !line.includes("at Object.apply"))
                .filter((line) => !line.includes("at Object.get"))
                .join("\n");
            
            const getParameterProxyHandler = {
              get(target, key) {
                try {
                  // Mitigate Chromium bug (#130)
                  if (typeof target[key] === "function") {
                    return target[key].bind(target);
                  }
                  return Reflect.get(target, key);
                } catch (err) {
                  err.stack = stripErrorStack(err.stack);
                  throw err;
                }
              },
              apply: function (target, thisArg, args) {
                const param = (args || [])[0];
                // UNMASKED_VENDOR_WEBGL
                if (param === 37445) return "Intel Inc.";
                // UNMASKED_RENDERER_WEBGL
                if (param === 37446) return "Intel(R) Iris(TM) Plus Graphics 640";
                try {
                  return Reflect.apply(target, thisArg, args);
                } catch (err) {
                  err.stack = stripErrorStack(err.stack);
                  throw err;
                }
              },
            };
            
            ["WebGLRenderingContext", "WebGL2RenderingContext"].forEach(function (ctx) {
              Object.defineProperty(window[ctx].prototype, "getParameter", {
                configurable: true,
                enumerable: false,
                writable: false,
                value: new Proxy(
                  window[ctx].prototype.getParameter,
                  getParameterProxyHandler
                ),
              });
            });
            // https://github.com/microlinkhq/browserless/blob/master/packages/goto/src/evasions/media-codecs.js
            try {
              /**
               * Input might look funky, we need to normalize it so e.g. whitespace isn't an issue for our spoofing.
               *
               * @example
               * video/webm; codecs="vp8, vorbis"
               * video/mp4; codecs="avc1.42E01E"
               * audio/x-m4a;
               * audio/ogg; codecs="vorbis"
               * @param {String} arg
               */
              const parseInput = (arg) => {
                const [mime, codecStr] = arg.trim().split(";");
                let codecs = [];
                if (codecStr && codecStr.includes('codecs="')) {
                  codecs = codecStr
                    .trim()
                    .replace('codecs="', "")
                    .replace('"', "")
                    .trim()
                    .split(",")
                    .filter((x) => !!x)
                    .map((x) => x.trim());
                }
                return { mime, codecStr, codecs };
              };
            
              /* global HTMLMediaElement */
              const canPlayType = {
                // Make toString() native
                get(target, key) {
                  // Mitigate Chromium bug (#130)
                  if (typeof target[key] === "function") {
                    return target[key].bind(target);
                  }
                  return Reflect.get(target, key);
                },
                // Intercept certain requests
                apply: function (target, ctx, args) {
                  if (!args || !args.length) {
                    return target.apply(ctx, args);
                  }
                  const { mime, codecs } = parseInput(args[0]);
                  // This specific mp4 codec is missing in Chromium
                  if (mime === "video/mp4") {
                    if (codecs.includes("avc1.42E01E")) {
                      return "probably";
                    }
                  }
                  // This mimetype is only supported if no codecs are specified
                  if (mime === "audio/x-m4a" && !codecs.length) {
                    return "maybe";
                  }
            
                  // This mimetype is only supported if no codecs are specified
                  if (mime === "audio/aac" && !codecs.length) {
                    return "probably";
                  }
                  // Everything else as usual
                  return target.apply(ctx, args);
                },
              };
              HTMLMediaElement.prototype.canPlayType = new Proxy(
                HTMLMediaElement.prototype.canPlayType,
                canPlayType
              );
            } catch (err) {}            
            "#
        }),
    )
    .await
    .unwrap();
}

async fn solve_incolumitas(driver: &WebDriver) {
    driver.goto("https://bot.incolumitas.com").await.unwrap();
    let page = driver.query(By::Id("formStuff")).first().await.unwrap();

    let username_element = page.find(By::Css(r#"[name="userName"]"#)).await.unwrap();
    username_element.click().await.unwrap();
    username_element.clear().await.unwrap();
    username_element.send_keys("bot3000").await.unwrap();

    // same stuff here
    let email_element = page.find(By::Css(r#"[name="eMail"]"#)).await.unwrap();
    email_element.click().await.unwrap();
    email_element.clear().await.unwrap();
    email_element.send_keys("bot3000@gmail.com").await.unwrap();

    // cookie element
    let cookie_element = page.find(By::Css(r#"[name="cookies"]"#)).await.unwrap();
    cookie_element.click().await.unwrap();
    cookie_element
        .send_keys("I want all the Cookies")
        .await
        .unwrap();

    // smol cat
    let el = driver.find(By::Id("smolCat")).await.unwrap();
    el.click().await.unwrap();
    // big cat
    let el = driver.find(By::Id("bigCat")).await.unwrap();
    el.click().await.unwrap();
    // submit
    let el = driver.find(By::Id("submit")).await.unwrap();
    el.click().await.unwrap();

    driver.accept_alert().await.unwrap();

    // wait for results to appear
    driver
        .query(By::Css("#tableStuff tbody tr .url"))
        .first()
        .await
        .unwrap();
    // just in case
    sleep(Duration::from_millis(100)).await;

    let p = page
        .query(By::Css("#tableStuff tbody tr .url"))
        .first()
        .await
        .unwrap();

    // now update both prices
    // by clicking on the "Update Price" button
    //  await page.waitForSelector('#updatePrice0');
    //  await page.click('#updatePrice0');
    //  await page.waitForFunction('!!document.getElementById("price0").getAttribute("data-last-update")');

    //  await page.waitForSelector('#updatePrice1');
    //  await page.click('#updatePrice1');
    //  await page.waitForFunction('!!document.getElementById("price1").getAttribute("data-last-update")');

    // Type in the search terms.
    //  elem_text.send_keys("selenium").await?;

    //  // Click the search button.
    //  let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
    //  elem_button.click().await?;

    //  // Look for header to implicitly wait for the page to load.
    //  driver.find(By::ClassName("firstHeading")).await?;
    //  assert_eq!(driver.title().await?, "Selenium - Wikipedia");

    // overwrite the existing text by selecting it
    // with the mouse with a triple click

    // const userNameInput = await page.$('[name="userName"]');
}

async fn configure_headful(driver: &WebDriver) {
    let dev_tools = ChromeDevTools::new(driver.handle.clone());

    println!("adding chrome evasions");
    dev_tools
  .execute_cdp_with_params(
      "Page.addScriptToEvaluateOnNewDocument",
      json!({
          "source": r#"
          // monkey patch Object.defineProperty
          const { defineProperty } = window.Object;
          window.Object.defineProperty = function (o, p, attrs) {
            if (o instanceof Error && o.stack && p === "stack") {
              return o;
            }
            return defineProperty.call(this, o, p, attrs);
          };
          // https://github.com/microlinkhq/browserless/blob/master/packages/goto/src/evasions/webgl-vendor.js
          // Remove traces of our Proxy ;-)
          const stripErrorStack = (stack) =>
            stack
              .split("\n")
              .filter((line) => !line.includes("at Object.apply"))
              .filter((line) => !line.includes("at Object.get"))
              .join("\n");
          
          const getParameterProxyHandler = {
            get(target, key) {
              try {
                // Mitigate Chromium bug (#130)
                if (typeof target[key] === "function") {
                  return target[key].bind(target);
                }
                return Reflect.get(target, key);
              } catch (err) {
                err.stack = stripErrorStack(err.stack);
                throw err;
              }
            },
            apply: function (target, thisArg, args) {
              const param = (args || [])[0];
              // UNMASKED_VENDOR_WEBGL
              if (param === 37445) return "Intel Inc.";
              // UNMASKED_RENDERER_WEBGL
              if (param === 37446) return "Intel(R) Iris(TM) Plus Graphics 640";
              try {
                return Reflect.apply(target, thisArg, args);
              } catch (err) {
                err.stack = stripErrorStack(err.stack);
                throw err;
              }
            },
          };
          
          ["WebGLRenderingContext", "WebGL2RenderingContext"].forEach(function (ctx) {
            Object.defineProperty(window[ctx].prototype, "getParameter", {
              configurable: true,
              enumerable: false,
              writable: false,
              value: new Proxy(
                window[ctx].prototype.getParameter,
                getParameterProxyHandler
              ),
            });
          });          
          "#
      }),
  )
  .await
  .unwrap();
}

// https://hmaker.github.io/selenium-detector/
async fn solve_selenium_detector(driver: &WebDriver) {
    driver
        .goto("https://hmaker.github.io/selenium-detector/")
        .await
        .unwrap();

    let token_v = driver
        .execute("return window.token;", Vec::new())
        .await
        .unwrap()
        .convert::<String>()
        .unwrap();

    let token = driver.find(By::Id("chromedriver-token")).await.unwrap();
    token.send_keys(token_v).await.unwrap();

    let token_v = driver
        .execute("return window.getAsyncToken();", Vec::new())
        .await
        .unwrap()
        .convert::<String>()
        .unwrap();
    println!("{:?}", token_v);

    let async_token = driver
        .find(By::Id("chromedriver-asynctoken"))
        .await
        .unwrap();
    async_token.send_keys(token_v).await.unwrap();

    let submit = driver.find(By::Id("chromedriver-test")).await.unwrap();
    submit.click().await.unwrap();
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    //  caps.add_chrome_option("remote-debugging-port", "9222");
    // caps.add_chrome_option("debuggerAddress", "127.0.0.1:9222")?;
    // caps.set_binary("/usr/lib/chromium/chromium")?;
    // caps.add_arg("--user-data-dir=/home/mira/.temp_webdriver")?;
    caps.add_arg("--autoplay-policy=user-gesture-required")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kAutoplayPolicy&ss=chromium
    caps.add_exclude_switch("enable-automation")?;
    caps.add_arg("--disable-blink-features=AutomationControlled")?; // https://blog.m157q.tw/posts/2020/09/11/bypass-cloudflare-detection-while-using-selenium-with-chromedriver/
                                                                    // caps.add_arg("--disable-cloud-import")?;
                                                                    // caps.add_arg("--disable-component-update")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kDisableComponentUpdate&ss=chromium
                                                                    // caps.add_arg("--disable-domain-reliability")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kDisableDomainReliability&ss=chromium
                                                                    // caps.add_arg(
                                                                    // "--disable-features=AudioServiceOutOfProcess,IsolateOrigins,site-per-process",
                                                                    // )?; // https://source.chromium.org/search?q=file:content_features.cc&ss=chromium
                                                                    // caps.add_arg("--disable-gesture-typing")?;
                                                                    // caps.add_arg("--disable-infobars")?;
                                                                    // caps.add_arg("--disable-notifications")?;
                                                                    // caps.add_arg("--disable-offer-store-unmasked-wallet-cards")?;
                                                                    // caps.add_arg("--disable-offer-upload-credit-cards")?;
                                                                    // caps.add_arg("--disable-print-preview")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kDisablePrintPreview&ss=chromium
                                                                    // caps.add_arg("--disable-setuid-sandbox")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kDisableSetuidSandbox&ss=chromium
                                                                    //                                            // cloudflare challenge doesn't pass with this disabled
                                                                    //                                            // caps.add_arg("--disable-site-isolation-trials")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kDisableSiteIsolation&ss=chromium
                                                                    // caps.add_arg("--disable-speech-api")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kDisableSpeechAPI&ss=chromium
                                                                    // caps.add_arg("--disable-tab-for-desktop-share")?;
                                                                    // caps.add_arg("--disable-translate")?;
                                                                    // caps.add_arg("--disable-voice-input")?;
                                                                    // caps.add_arg("--disable-wake-on-wifi")?;
                                                                    // caps.add_arg("--enable-async-dns")?;
                                                                    // caps.add_arg("--enable-simple-cache-backend")?;
                                                                    // caps.add_arg("--enable-tcp-fast-open")?;
                                                                    // caps.add_arg("--enable-webgl")?;
                                                                    // caps.add_arg("--force-webrtc-ip-handling-policy=default_public_interface_only")?;
                                                                    // caps.add_arg("--ignore-gpu-blocklist")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kIgnoreGpuBlocklist&ss=chromium
                                                                    // caps.add_arg("--no-default-browser-check")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kNoDefaultBrowserCheck&ss=chromium
                                                                    // caps.add_arg("--no-pings")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kNoPings&ss=chromium
    caps.add_arg("--no-sandbox")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kNoSandbox&ss=chromium
                                   // caps.add_arg("--no-zygote")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kNoZygote&ss=chromium
                                   // caps.add_arg("--prerender-from-omnibox=disabled")?;
    caps.add_arg("--use-gl=desktop")?; // https://source.chromium.org/search?q=lang:cpp+symbol:kUseGl&ss=chromium
    caps.add_arg("--remote-debugging-port=9222")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--start-maximized")?;
    // caps.add_arg("--ozone-platform-hint=wayland")?;
    // caps.set_headless()?;
    caps.set_no_sandbox()?;
    caps.set_disable_gpu()?;
    caps.set_disable_dev_shm_usage()?;
    // caps.add_arg("--headless=new")?;
    // caps.set_debugger_address("127.0.0.1:9222")?;
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // configure_headless(&driver).await;
    configure_headful(&driver).await;
    // solve_incolumitas(&driver).await;
    solve_selenium_detector(&driver).await;

    // driver.goto("https://detect.azerpas.com/").await?;
    // driver.got
    // driver
    //     .screenshot(Path::new("./nowsecure1.png"))
    //     .await
    //     .unwrap();
    // sleep(Duration::from_secs(2)).await;
    // driver
    //     .screenshot(Path::new("./nowsecure2.png"))
    //     .await
    //     .unwrap();
    // sleep(Duration::from_secs(4)).await;
    // driver
    //     .screenshot(Path::new("./nowsecure3.png"))
    //     .await
    //     .unwrap();
    // sleep(Duration::from_secs(8)).await;
    // driver.screenshot(Path::new("./creepjs.png")).await.unwrap();

    // Find element from element.
    // let elem_text = driver.find(By::Css(".px-3 > h1:nth-child(1)")).await?.text().await?;
    // println!("{}", elem_text);

    // sleep(Duration::from_secs(360)).await;

    // Always explicitly close the browser.
    // driver.quit().await?;

    Ok(())
}

// // let's set up the sequence of steps we want the browser to take
// #[tokio::main]
// async fn main() -> Result<(), fantoccini::error::CmdError> {
//     let mut caps = serde_json::map::Map::new();
//     let opts = serde_json::json!({
//         "args": ["--remote-debugging-port=9222", "--headless", "--disable-gpu", "--no-sandbox", "--disable-dev-shm-usage"],
//     });
//     caps.insert("goog:chromeOptions".to_string(), opts);

//     let c = ClientBuilder::rustls()
//         .capabilities(caps)
//         .connect("http://localhost:4444")
//         .await
//         .expect("failed to connect to WebDriver");

//     // *[@id="mount_0_0_Cq"]/div/div[1]/div/div[3]/div/div/div/div[1]/div[1]/div/div/div[4]/div/div/div/div/div/div/div/div/div[3]

//     // first, go to the Wikipedia page for Foobar
//     c.goto("https://nowsecure.nl").await?;
//     let url = c.current_url().await?;

//     Ok(())
//     // println!("{}", url);

//     // let h1 = c.find(Locator::Css(".px-3 > h1:nth-child(1)")).await?;
//     // let h1_text = h1.text().await?;
//     // println!("{}", h1_text);

//     // c.close().await
// }
