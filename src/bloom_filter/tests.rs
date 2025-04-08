use crate::bloom_filter::BloomFilter;

#[test]
fn test_filter_size() {
    let filter = BloomFilter::<u64>::new(100000, 0.2);

    assert_eq!(filter.filter.len(), 41873);
    assert_eq!(filter.hash_functions, 3);

    let filter = BloomFilter::<u64>::new(10000, 0.2);

    assert_eq!(filter.filter.len(), 4188);
    assert_eq!(filter.hash_functions, 3);

    let filter = BloomFilter::<u64>::new(100000, 0.1);

    assert_eq!(filter.filter.len(), 59907);
    assert_eq!(filter.hash_functions, 4);

    let filter = BloomFilter::<u64>::new(10000, 0.1);

    assert_eq!(filter.filter.len(), 5991);
    assert_eq!(filter.hash_functions, 4);
}

#[test]
fn test_addition_and_finding() {
    let mut filter = BloomFilter::new(100, 0.2);

    for i in 0..100 {
        filter.add(i);
    }

    for i in 0..100 {
        assert!(filter.contains(&i));
    }

    let mut positive = 0.0;
    let mut false_positive = 0.0;

    for i in 100..1000 {
        if filter.contains(&i) {
            false_positive += 1.0;
        } else {
            positive += 1.0;
        }
    }

    let ratio = false_positive / (positive + false_positive);
    let rounded_ratio = (ratio * 100.0f32).round() / 100.0;

    assert_eq!(rounded_ratio, 0.2);
}
