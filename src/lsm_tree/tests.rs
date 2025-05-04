use crate::lsm_tree::LsmTree;

#[test]
fn test_initialization_creates_empty_directory() {
    let path = "target/test_initialization_creates_empty_directory";

    let _ = std::fs::remove_dir_all(path);

    let _ = lsm_three("test_initialization_creates_empty_directory");

    let expected_content = vec![format!("{path}/state"), format!("{path}/level0")];

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
    let mut tree = lsm_three("test_memtable_never_exceeds_configured_size");

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

    for i in 0..1000 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    tree.flush().unwrap();

    // Since search should go in reversed chronological order and key_18 should appear in a #0
    // SSTable, which will be reached the last, we expect that no other SSTables will be read
    // (despite the fact that some SSTable's inner bloom filter can return false-positive).
    let key = "key_18".to_string();

    for table in 1..10 {
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
    let mut tree = lsm_three("test_no_reads_in_unrequired_ss_tables");

    for i in 0..1000 {
        tree.insert(format!("key_{i}"), format!("value_{i}"))
            .unwrap();
    }

    tree.flush().unwrap();

    let tree =
        LsmTree::<String, String>::load("target/test_no_reads_in_unrequired_ss_tables".to_string())
            .unwrap();

    let value = tree.get(&"key_12".to_string()).unwrap();

    assert_eq!(value, Some("value_12".to_string()));
}

fn lsm_three(test_name: &str) -> LsmTree<String, String> {
    LsmTree::new(format!("target/{test_name}"), 100, 10).unwrap()
}
