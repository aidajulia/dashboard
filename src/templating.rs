use hbs::{HandlebarsEngine, DirectorySource};
use utils::from_config;


pub fn init_templating() -> HandlebarsEngine {
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new(&from_config("DASHBOARD_DASHBOARDS_DIR_PATH"), &".html".to_owned())));
    hbse.add(Box::new(DirectorySource::new(&"src/templates/", &".html")));
    if let Err(r) = hbse.reload() {
        panic!("{}", r);
    }
    hbse
}
