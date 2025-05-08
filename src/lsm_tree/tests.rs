use crate::lsm_tree::LsmTree;

#[test]
fn test_initialization_creates_empty_directory() {
    let path = "target/test_initialization_creates_empty_directory";

    let _ = std::fs::remove_dir_all(path);

    let _ = lsm_three("test_initialization_creates_empty_directory");

    let expected_content = vec![
        format!("{path}/state"),
        format!("{path}/level0"),
        format!("{path}/level1"),
    ];

    let actual_content: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path().to_string_lossy().into_owned())
        .collect();

    assert_eq!(actual_content, expected_content);
}

#[test]
fn test_simple_insert_and_get() {
    let mut tree = lsm_three("test_simple_insert_and_get");

    let key = "Hello".to_string();
    let value = "World".to_string();

    tree.insert(key.clone(), value.clone()).unwrap();

    let found_value = tree.get(&key).unwrap();

    assert_eq!(found_value, Some(value));
}

#[test]
fn test_memtable_never_exceeds_configured_size_while_all_data_is_accessible() {
    let mut tree =
        lsm_three("test_memtable_never_exceeds_configured_size_while_all_data_is_accessible");

    for i in 0..1000 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    assert_eq!(tree.map.len(), 100);

    for i in 0..1000 {
        let value = tree.get(&format!("key_{i}")).unwrap();
        let expected_value = format!("value_{i}");
        assert_eq!(value, Some(expected_value));
    }
}

#[test]
fn test_no_reads_in_unrequired_ss_tables() {
    let mut tree = lsm_three("test_no_reads_in_unrequired_ss_tables");

    for i in 0..800 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    tree.flush().unwrap();

    // Since search should go in reversed chronological order and key_18 should appear in a #0
    // SSTable, which will be reached the last, we expect that no other SSTables will be read
    // (despite the fact that some SSTable's inner bloom filter can return false-positive).
    let key = "key_18".to_string();

    for table in 1..8 {
        std::fs::remove_file(format!(
            "target/test_no_reads_in_unrequired_ss_tables/level0/{table}.data"
        ))
        .unwrap();
    }

    assert!(tree.get(&key).unwrap().is_some());

    // Just to ensure that other ss tables are deleted
    let key = "key_700".to_string();
    assert!(tree.get(&key).is_err());
}

#[test]
fn load_tree() {
    let mut tree = lsm_three("load_tree");

    for i in 0..800 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    tree.flush().unwrap();

    let tree = LsmTree::<String, String>::load("target/load_tree".to_string()).unwrap();

    let value = tree.get(&"key_12".to_string()).unwrap();

    assert_eq!(value, Some("value_12".to_string()));
}

#[test]
fn test_compaction_moves_data_to_level_1() {
    let mut tree = lsm_three("test_compaction_moves_data_to_level_1");

    // Produces 5 saved SS Tables, so 15 files
    for i in 0..500 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }
    tree.flush().unwrap();

    tree.compact().unwrap();

    assert_eq!(
        std::fs::read_dir("target/test_compaction_moves_data_to_level_1/level0")
            .unwrap()
            .count(),
        0
    );

    assert_eq!(
        std::fs::read_dir("target/test_compaction_moves_data_to_level_1/level1")
            .unwrap()
            .count(),
        3 // for idx, filter and data
    );
}

#[test]
fn test_after_compaction_data_is_still_accessible() {
    let mut tree = lsm_three("test_after_compaction_data_is_still_accessible");

    for i in 0..500 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }
    tree.flush().unwrap();
    tree.compact().unwrap();

    let value = tree.get(&"key_18".to_string()).unwrap();
    assert_eq!(value, Some("value_18".to_string()));
}

#[test]
fn test_compaction_leaves_more_recent_key_value() {
    let mut tree = lsm_three("test_compaction_leaves_more_recent_key_value");

    for i in 0..5 {
        tree.insert("key".to_string(), format!("v{i}")).unwrap();
        tree.flush().unwrap();
    }

    tree.compact().unwrap();

    let value = tree.get(&"key".to_string()).unwrap();

    assert_eq!(value, Some("v4".to_string()));
}

#[test]
fn test_delete_if_key_exists() {
    let mut tree = lsm_three("test_delete_if_key_exists");

    for i in 0..1500 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    let value = tree.get(&"key_12".to_string()).unwrap();
    assert_eq!(value, Some("value_12".to_string()));

    let value = tree.delete("key_12".to_string()).unwrap();
    assert_eq!(value, Some("value_12".to_string()));

    let value = tree.get(&"key_12".to_string()).unwrap();
    assert_eq!(value, None);
}

#[test]
fn test_skip_if_key_does_not_exist() {
    let mut tree = lsm_three("test_skip_if_key_does_not_exist");

    for i in 0..1500 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    let value = tree.get(&"not_exists".to_string()).unwrap();
    assert!(value.is_none());

    let value = tree.delete("not_exists".to_string()).unwrap();
    assert!(value.is_none());
}

#[test]
fn test_compaction_works_with_deletion() {
    let mut tree = lsm_three("test_compaction_works_with_deletion");

    for i in 0..500 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    tree.insert("some_value".to_string(), "some_value".to_string())
        .unwrap();

    for i in 500..1000 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    tree.delete("some_value".to_string()).unwrap();

    for i in 1000..1500 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    tree.flush().unwrap();

    tree.compact().unwrap();

    assert!(tree.get(&"some_value".to_string()).unwrap().is_none());
}

fn lsm_three(test_name: &str) -> LsmTree<String, String> {
    LsmTree::new(format!("target/{test_name}"), 100, 10, 10).unwrap()
}
