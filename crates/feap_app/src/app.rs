/// [`App`] is the primary API for writing user applications.
/// ```
pub struct App {
    /// The function that will manage the app's lifecycle.
    pub(crate) runner: RunnerFn,
}

impl Default for App {
    fn default() -> Self {
        App::empty()
    }
}

impl App {
    /// Creates a new [`App`] with some default structure to enable core engine features
    /// ```
    pub fn new() -> App {
        App::default()
    }

    /// Creates a new empty [`App`] with minimal configuration
    ///
    pub fn empty() -> App {
        App {
            runner: Box::new(run_once),
        }
    }

    /// Runs the [`App`], by calling its [runner].
    ///
    pub fn run(&mut self) {
        #[cfg(feature = "trace")]
        let _feap_app_run_span = info_span!("feap_app").entered();

        let runner = core::mem::replace(&mut self.runner, Box::new(run_once));
        let app = std::mem::take(self);
        (runner)(app);
    }
}

type RunnerFn = Box<dyn FnOnce(App) -> AppExit>;

fn run_once(_app: App) -> AppExit {
    println!("Running app once...");
    AppExit::Success
}

/// A [`BufferedEvent`] that indicates the [`App`] should exit.
///
pub enum AppExit {
    /// [`App`] exited successfully.
    Success,
}
