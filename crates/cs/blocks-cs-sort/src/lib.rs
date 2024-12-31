pub fn quicksort<T: Ord + Clone>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }
    
    let pivot_idx = partition(arr);
    let len = arr.len();
    
    quicksort(&mut arr[0..pivot_idx]);
    quicksort(&mut arr[pivot_idx + 1..len]);
}

fn partition<T: Ord + Clone>(arr: &mut [T]) -> usize {
    let len = arr.len();
    let pivot_idx = len - 1;
    let pivot = arr[pivot_idx].clone();
    let mut i = 0;
    
    for j in 0..len - 1 {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    
    arr.swap(i, pivot_idx);
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_array() {
        let mut arr: Vec<i32> = vec![];
        quicksort(&mut arr);
        assert_eq!(arr, vec![]);
    }

    #[test]
    fn test_single_element() {
        let mut arr = vec![1];
        quicksort(&mut arr);
        assert_eq!(arr, vec![1]);
    }

    #[test]
    fn test_sorted_array() {
        let mut arr = vec![1, 2, 3, 4, 5];
        quicksort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_reverse_sorted_array() {
        let mut arr = vec![5, 4, 3, 2, 1];
        quicksort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_random_array() {
        let mut arr = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        quicksort(&mut arr);
        assert_eq!(arr, vec![1, 1, 2, 3, 3, 4, 5, 5, 5, 6, 9]);
    }
}
