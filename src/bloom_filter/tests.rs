use crate::bloom_filter::BloomFilter;
use std::collections::BTreeSet;

#[test]
fn test_filter_size() {
    let filter = BloomFilter::<u64>::new(100000, 0.2);

    assert_eq!(filter.filter.byte_size(), 41873);
    assert_eq!(filter.hash_functions, 3);

    let filter = BloomFilter::<u64>::new(10000, 0.2);

    assert_eq!(filter.filter.byte_size(), 4188);
    assert_eq!(filter.hash_functions, 3);

    let filter = BloomFilter::<u64>::new(100000, 0.1);

    assert_eq!(filter.filter.byte_size(), 59907);
    assert_eq!(filter.hash_functions, 4);

    let filter = BloomFilter::<u64>::new(10000, 0.1);

    assert_eq!(filter.filter.byte_size(), 5991);
    assert_eq!(filter.hash_functions, 4);
}

#[test]
fn test_addition_and_finding() {
    let false_positives_probability = 0.1;

    let mut filter = BloomFilter::new(100, false_positives_probability);

    let values: BTreeSet<u16> = (0..10000).step_by(100).collect();

    for i in values.clone() {
        filter.add(i);
    }

    let mut positive = 0;
    let mut negative = 0;
    let mut false_positive = 0;

    for i in 0..10000 {
        if filter.contains(&i) && values.contains(&i) {
            positive += 1;
        } else if filter.contains(&i) {
            false_positive += 1;
        } else {
            negative += 1;
        }
    }

    assert_eq!(positive, 100);

    let ratio = false_positive as f64 / (negative as f64 + false_positive as f64);

    let ratio_diff = (ratio - false_positives_probability).abs();

    assert!(ratio_diff < 0.01);
}
