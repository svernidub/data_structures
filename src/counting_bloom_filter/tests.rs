use super::CountingBloomFilter;

#[test]

fn test_filter() {
    let mut filter =
        CountingBloomFilter::with_planned_capacity_and_false_positives_probability(100, 0.2);

    for i in 0..100 {
        filter.add(&i);
    }

    for i in 0..100 {
        assert!(filter.contains(&i));
    }

    let mut positive = 0;
    let mut false_positive = 0;

    for i in 100..1000 {
        if filter.contains(&i) {
            false_positive += 1;
        } else {
            positive += 1;
        }
    }

    assert_eq!((positive, false_positive), (674, 226));

    filter.remove(&5);

    assert!(!filter.contains(&5));
}
