use hbs::{HandlebarsEngine, DirectorySource};


pub fn init_templating() -> HandlebarsEngine {
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new(&"src/templates/", &".html")));
    // TODO: register helper for reverse url
    // (https://github.com/iron/router/blob/master/examples/url_for.rs)
    // hbse.handlebars_mut().register_helper("helper", my_helper);
    if let Err(r) = hbse.reload() {
        panic!("{}", r);
    }
    hbse
}
