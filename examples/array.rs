use cordwood::db::{DBConfig, WALConfig, DB};

fn main() {
    let cfg = DBConfig::builder().wal(WALConfig::builder().max_revisions(10).build());
    {
        let db = DB::new("array_db", &cfg.clone().truncate(true).build()).unwrap();
        db.new_writebatch()
            .array_set(0, 100)
            .unwrap()
            .array_set(2, 102)
            .unwrap()
            .commit();
        println!("0 => {}", db.array_get(0).unwrap());
        db.array_dump(&mut std::io::stdout()).unwrap();
    }
    {
        let db = DB::new("array_db", &cfg.truncate(false).build()).unwrap();
        db.array_dump(&mut std::io::stdout()).unwrap();
    }
}
