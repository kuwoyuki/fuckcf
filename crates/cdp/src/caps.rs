use fxhash::FxHashMap;

// args 	list of strings 	 	List of command-line arguments to use when starting Chrome. Arguments with an associated value should be separated by a '=' sign (e.g., ['start-maximized', 'user-data-dir=/tmp/temp_profile']). See here for a list of Chrome arguments.
// binary	string 	 	Path to the Chrome executable to use (on Mac OS X, this should be the actual binary, not just the app. e.g., '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome')
// extensions	list of strings 	 	A list of Chrome extensions to install on startup. Each item in the list should be a base-64 encoded packed Chrome extension (.crx)
// localState	dictionary	 	A dictionary with each entry consisting of the name of the preference and its value. These preferences are applied to the Local State file in the user data folder.
// prefs	dictionary	 	A dictionary with each entry consisting of the name of the preference and its value. These preferences are only applied to the user profile in use. See the 'Preferences' file in Chrome's user data directory for examples.
// detach	boolean	false 	If false, Chrome will be quit when ChromeDriver is killed, regardless of whether the session is quit. If true, Chrome will only be quit if the session is quit (or closed). Note, if true, and the session is not quit, ChromeDriver cannot clean up the temporary user data directory that the running Chrome instance is using.
// debuggerAddress	string
// An address of a Chrome debugger server to connect to, in the form of <hostname/ip:port>, e.g. '127.0.0.1:38947'
// excludeSwitches	list of strings 	 	List of Chrome command line switches to exclude that ChromeDriver by default passes when starting Chrome.  Do not prefix switches with --.
// minidumpPath 	string 	 	Directory to store Chrome minidumps . (Supported only on Linux.)
// mobileEmulation	dictionary	 	A dictionary with either a value for “deviceName,” or values for “deviceMetrics” and “userAgent.” Refer to Mobile Emulation for more information.
// perfLoggingPrefs	dictionary	 	 An optional dictionary that specifies performance logging preferences. See below for more information.
// windowTypes	 list of strings	 	A list of window types that will appear in the list of window handles. For access to <webview> elements, include "webview" in this list.

// #[derive(Clone)]
pub struct Capabilities {
    pub args: Vec<String>,
    pub binary: String,
    pub extensions: Vec<String>, // todo
    pub local_state: FxHashMap<String, String>,
    pub detach: bool,
    pub debugger_address: String,
    pub launch: bool,
}

impl Capabilities {
    pub fn new() -> Self {
        Capabilities {
            args: Vec::new(),
            binary: "/usr/bin/chrome".to_string(),
            extensions: Vec::new(),
            local_state: FxHashMap::default(),
            detach: true,
            debugger_address: String::new(),
            launch: true,
        }
    }

    pub fn add_arg(&mut self, arg: &str) {
        let arg_string = arg.to_string();
        if !self.args.contains(&arg_string) {
            self.args.push(arg_string);
        }
    }

    /// Set the path to chrome binary to use.
    pub fn set_binary(&mut self, path: &str) {
        // check if it exists?
        self.binary = path.to_string()
    }

    pub fn disable_launch(&mut self) {
        self.launch = false
    }
}
