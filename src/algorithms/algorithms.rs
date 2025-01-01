pub fn linear_search<T: PartialEq>(arr: &[T], target: &T) -> Option<usize> {
    for (index, item) in arr.iter().enumerate() {
        if item == target {
            return Some(index);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_search_found() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(linear_search(&arr, &5), Some(2));
    }

    #[test]
    fn test_linear_search_not_found() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(linear_search(&arr, &4), None);
    }

    #[test]
    fn test_linear_search_empty_array() {
        let arr: Vec<i32> = vec![];
        assert_eq!(linear_search(&arr, &1), None);
    }

    #[test]
    fn test_linear_search_first_element() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(linear_search(&arr, &1), Some(0));
    }

    #[test]
    fn test_linear_search_last_element() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(linear_search(&arr, &9), Some(4));
    }

    #[test]
    fn test_linear_search_with_duplicates() {
        let arr = vec![1, 3, 5, 5, 7, 9];
        assert_eq!(linear_search(&arr, &5), Some(2)); // Returns first occurrence
    }

    #[test]
    fn test_linear_search_strings() {
        let arr = vec!["apple", "banana", "orange"];
        assert_eq!(linear_search(&arr, &"banana"), Some(1));
    }

    #[test]
    fn test_linear_search_single_element() {
        let arr = vec![42];
        assert_eq!(linear_search(&arr, &42), Some(0));
        assert_eq!(linear_search(&arr, &43), None);
    }
}