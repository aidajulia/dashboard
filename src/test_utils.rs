use db;

pub fn upsert_dashboard(db: &db::Db, dashboard_name: &str) -> db::Dashboard {
    let dashboard = db::Dashboard::new(
        dashboard_name.to_string(),
        "login@email.com".to_string(),
        "2x4".to_string(),
    );
    db.upsert_dashboard(&dashboard).unwrap();
    db.get_dashboard(dashboard_name).unwrap().unwrap()
}
