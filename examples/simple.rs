use cordwood::db::{DBConfig, WALConfig, DB};

fn main() {
    let cfg = DBConfig::builder().wal(WALConfig::builder().max_revisions(10).build());
    /*
    {
        let db = DB::new("simple_db", &cfg.clone().truncate(true).build()).unwrap();
        db.new_writebatch()
            .set_balance(b"ted", 10.into())
            .unwrap()
            .set_code(b"ted", b"smart contract byte code here!")
            .unwrap()
            .set_nonce(b"ted", 10086)
            .unwrap()
            .set_state(b"ted", b"x", b"1".to_vec())
            .unwrap()
            .set_state(b"ted", b"y", b"2".to_vec())
            .unwrap()
            .commit();
    }
    {
        let db = DB::new("simple_db", &cfg.clone().truncate(false).build()).unwrap();
        print_states(&db);
        db.new_writebatch()
            .set_state(b"alice", b"z", b"999".to_vec())
            .unwrap()
            .commit();
        print_states(&db);
    }
    {
        let db = DB::new("simple_db", &cfg.truncate(false).build()).unwrap();
        print_states(&db);
        let mut stdout = std::io::stdout();
        let mut acc = None;
        db.dump(&mut stdout).unwrap();
        db.dump_account(b"ted", &mut stdout).unwrap();
        db.new_writebatch().delete_account(b"ted", &mut acc).unwrap();
        assert!(acc.is_some());
        print_states(&db);
        db.dump_account(b"ted", &mut stdout).unwrap();
        db.new_writebatch().delete_account(b"nobody", &mut acc).unwrap();
        assert!(acc.is_none());
    }
    */
}
