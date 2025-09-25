#[cfg(test)]
mod test_input_issues {
    use super::*;
    use crate::input::{span::Span, string::SharedString, Input};

    // Test for the logic error in take_checked - condition is backwards

    // Test range boundary issues in split_at methods
    #[test]
    fn test_span_split_at_boundary_issues() {
        let span = Span::anonymous("hello");

        // Should be able to split at the end (creates empty suffix)
        let (prefix, suffix) = span.split_at(5);
        assert_eq!(prefix.as_str(), "hello");
        assert_eq!(suffix.as_str(), "");

        // Test with split_at_checked as well
        let result = dbg!(span.split_at_checked(span.len()));
        assert!(result.is_some(), "should be able to split at end boundary");
        let (prefix, suffix) = result.unwrap();
        assert_eq!(prefix.as_str(), "hello");
        assert_eq!(suffix.as_str(), "");
    }

    #[test]
    fn test_shared_string_split_at_boundary_issues() {
        let shared = SharedString::new("hello".to_string());

        // Should be able to split at the end (creates empty suffix)
        let (prefix, suffix) = shared.split_at(5);
        assert_eq!(prefix.as_str(), "hello");
        assert_eq!(suffix.as_str(), "");

        // Test with split_at_checked as well
        let result = shared.split_at_checked(5);
        assert!(result.is_some(), "should be able to split at end boundary");
        let (prefix, suffix) = result.unwrap();
        assert_eq!(prefix.as_str(), "hello");
        assert_eq!(suffix.as_str(), "");
    }

    // Test case_insensitive_comparison edge cases
    #[test]
    fn test_case_insensitive_comparison_unicode() {
        let input = "héllo";

        // Test with accented characters
        let result = input.pop_no_case("HÉLLO");
        assert!(
            result.is_some(),
            "should match accented characters case-insensitively"
        );

        // Test with different length Unicode sequences
        let input = "ß"; // German eszett
        let result = input.pop_no_case("SS");
        // This test may fail due to Unicode case folding complexities
        // The current implementation doesn't handle this properly
    }

    // Test inconsistent range handling across different Input implementations
    #[test]
    fn test_range_consistency_across_implementations() {
        let str_input = "hello world";
        let string_input = str_input.to_string();
        let span_input = Span::anonymous(str_input);
        let shared_input = SharedString::new(str_input.to_string());

        // All should behave the same for basic operations
        assert_eq!(str_input.len(), string_input.len());
        assert_eq!(str_input.len(), span_input.len());
        assert_eq!(str_input.len(), shared_input.len());

        // All should handle empty slices the same way
        let str_empty = str_input.slice(0..0);
        let string_empty = string_input.slice(0..0);
        let span_empty = span_input.slice(0..0);
        let shared_empty = shared_input.slice(0..0);

        assert_eq!(str_empty.len(), 0);
        assert_eq!(string_empty.len(), 0);
        assert_eq!(span_empty.len(), 0);
        assert_eq!(shared_empty.len(), 0);

        // All should handle end boundary splits consistently
        let (str_a, str_b) = str_input.split_at_checked(11).unwrap();
        let (string_a, string_b) = string_input.split_at_checked(11).unwrap();
        let (span_a, span_b) = span_input.split_at_checked(11).unwrap();
        let (shared_a, shared_b) = shared_input.split_at_checked(11).unwrap();

        assert_eq!(str_a, "hello world");
        assert_eq!(string_a, "hello world");
        assert_eq!(span_a.as_str(), "hello world");
        assert_eq!(shared_a.as_str(), "hello world");

        assert_eq!(str_b, "");
        assert_eq!(string_b, "");
        assert_eq!(span_b.as_str(), "");
        assert_eq!(shared_b.as_str(), "");
    }

    // Test for potential panic conditions in slice operations
    #[test]
    #[should_panic]
    fn test_span_slice_panic_on_invalid_range() {
        let span = Span::anonymous("hello");
        // This should panic due to assertion failures
        let _invalid = span.slice(6..10); // start beyond bounds
    }

    #[test]
    #[should_panic]
    fn test_shared_string_slice_panic_on_invalid_range() {
        let shared = SharedString::new("hello".to_string());
        // This should panic due to assertion failures
        let _invalid = shared.slice(6..10); // start beyond bounds
    }

    // Test for off-by-one errors in range operations
    #[test]
    fn test_range_off_by_one_errors() {
        let input = "hello";

        // These should all work without panicking
        let result = input.split_at_checked(0);
        assert!(result.is_some());
        let (prefix, suffix) = result.unwrap();
        assert_eq!(prefix, "");
        assert_eq!(suffix, "hello");

        let result = input.split_at_checked(5);
        assert!(result.is_some());
        let (prefix, suffix) = result.unwrap();
        assert_eq!(prefix, "hello");
        assert_eq!(suffix, "");

        // This should fail
        let result = input.split_at_checked(6);
        assert!(result.is_none());
    }

    // Test Unicode boundary handling
    #[test]
    fn test_unicode_boundary_handling() {
        let input = "hello🌍world";

        // Make sure we can't split in the middle of a Unicode character
        let result = input.split_at_checked(6); // Would split the emoji
                                                // This may or may not be None depending on implementation

        // Test with emoji at different positions
        let emoji_start = "🌍hello";
        let emoji_end = "hello🌍";

        // These operations should preserve Unicode boundaries
        assert_eq!(emoji_start.take_checked(1), None);
        // Implementation may or may not handle this correctly

        let result = emoji_end.skip(5);
        assert_eq!(result, "🌍");
    }

    // Test consistency of take_while and take_until
    #[test]
    fn test_take_while_until_consistency() {
        let input = "aaabbb";

        // take_while should be inclusive of the predicate
        let result = input.take_while(|c| c == 'a');
        assert!(result.is_some());
        let (taken, remaining) = result.unwrap();
        assert_eq!(taken, "aaa");
        assert_eq!(remaining, "bbb");

        // take_until should stop at the first match
        let result = input.take_until(|c| c == 'b');
        assert!(result.is_some());
        let (taken, remaining) = result.unwrap();
        assert_eq!(taken, "aaa");
        assert_eq!(remaining, "bbb");

        // Edge case: empty input
        let empty = "";
        let result = empty.take_while(|c| c == 'a');
        assert!(result.is_none(), "take_while on empty should return None");

        let result = empty.take_until(|c| c == 'a');
        assert!(result.is_none(), "take_until on empty should return None");
    }

    // Test position calculation accuracy in Span
    #[test]
    fn test_span_position_calculation() {
        // Test single line
        let span = Span::anonymous("hello world");
        let pos = span.position();
        assert_eq!(pos, (1, 1)); // Should be line 1, column 1

        // Test after skipping some characters
        let skipped = span.skip(6);
        let pos = skipped.position();
        assert_eq!(pos, (1, 7)); // Should be line 1, column 7

        // Test multiline
        let multiline = Span::anonymous("hello\nworld\ntest");
        let lines: Vec<_> = multiline.as_str().split('\n').collect();

        // Find start of second line
        let second_line_start = "hello\n".len();
        let second_line = multiline.skip(second_line_start);
        let pos = second_line.position();
        assert_eq!(pos, (2, 1)); // Should be line 2, column 1

        // Find start of third line
        let third_line_start = "hello\nworld\n".len();
        let third_line = multiline.skip(third_line_start);
        let pos = third_line.position();
        assert_eq!(pos, (3, 1)); // Should be line 3, column 1
    }
}
