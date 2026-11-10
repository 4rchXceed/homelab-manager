use utils::config::{parse_file_size, parse_time};

#[test]
pub fn test_parse_time() {
    let test_cases = vec![
        ("1y", 365 * 24 * 60 * 60),
        ("2mo", 2 * 30 * 24 * 60 * 60),
        ("3w", 3 * 7 * 24 * 60 * 60),
        ("4d", 4 * 24 * 60 * 60),
        ("5h", 5 * 60 * 60),
        ("6m", 6 * 60),
        ("7s", 7),
        (
            "1y2mo3w4d5h6m7s",
            365 * 24 * 60 * 60
                + 2 * 30 * 24 * 60 * 60
                + 3 * 7 * 24 * 60 * 60
                + 4 * 24 * 60 * 60
                + 5 * 60 * 60
                + 6 * 60
                + 7,
        ),
    ];

    for (input, expected) in test_cases {
        let result = parse_time(input);
        println!("Testing parse_time('{}') => {:?}", input, result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
pub fn test_parse_file_size() {
    let test_cases = vec![
        ("1b", 1),
        ("1kb", 1024),
        ("1mb", 1024 * 1024),
        ("1gb", 1024 * 1024 * 1024),
        ("1tb", 1024 * 1024 * 1024 * 1024),
        ("1.398GB", (1501091072) as usize), // 1.39GB in bytes
    ];

    for (input, expected) in test_cases {
        let result = parse_file_size(input);
        println!("Testing parse_file_size('{}') => {:?}", input, result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
    assert!(parse_file_size("1.5b").is_err()); // Not an integer
    assert!(parse_file_size("-1tb").is_err()); // Not an integer
    assert!(parse_file_size("-9999999999999tb").is_err()); // Not an integer
}
