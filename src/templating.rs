use hbs::{HandlebarsEngine, DirectorySource};


pub fn init_templating() -> HandlebarsEngine {
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new(&"src/templates/", &".html")));
    if let Err(r) = hbse.reload() {
        panic!("{}", r);
    }
    hbse
}
