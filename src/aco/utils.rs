// May be used in the future, currently unusable because does not duplicate zeroes around path
#[allow(dead_code)]
pub fn path_to_slices(path: &mut [usize]) -> Vec<&mut [usize]> {
    use yoos::iter::FindAll;

    let zero_finder: Vec<usize> = path.iter().find_all(&0).collect();

    let mut paths = Vec::with_capacity(zero_finder.len());

    for (&idx1, &idx2) in zero_finder.iter().zip(zero_finder.iter().skip(1)) {
        unsafe {
            // SAFETY: The ranges are non-overlapping
            let start = path.as_mut_ptr().add(idx1 + 1);
            let len = (idx2 - idx1) - 1;
            let slice = std::slice::from_raw_parts_mut(start, len);
            paths.push(slice)
        }
    }

    paths
}

pub fn path_to_routes(path: &[usize]) -> Vec<Vec<usize>> {
    let mut paths = Vec::new();
    for &i in path {
        if i == 0 {
            paths.push(Vec::new());
        } else {
            let last: &mut Vec<usize> = paths.last_mut().unwrap();
            last.push(i);
        }
    }

    paths
}
