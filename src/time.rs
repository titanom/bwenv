use std::time::SystemTime;

pub fn is_date_older_than_n_seconds(unix_millis: u64, n_seconds: &u64) -> bool {
    let date_seconds = unix_millis / 1000;

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");

    let threshold_time = current_time.as_secs() - n_seconds;

    date_seconds < threshold_time
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::time::{Duration, SystemTime};
//
//     #[test]
//     fn date_is_older_than_n_seconds() {
//         let slightly_more_than_ten_seconds_ago = SystemTime::now()
//             .checked_sub(Duration::from_secs(11))
//             .expect("Failed to calculate time")
//             .duration_since(SystemTime::UNIX_EPOCH)
//             .expect("Time went backwards")
//             .as_secs()
//             * 1000; // Convert to milliseconds
//
//         assert!(is_date_older_than_n_seconds(
//             slightly_more_than_ten_seconds_ago,
//             &10
//         ));
//     }
//
//     #[test]
//     fn date_is_not_older_than_n_seconds() {
//         let five_seconds_ago = SystemTime::now()
//             .checked_sub(Duration::from_secs(5))
//             .expect("Failed to calculate time")
//             .duration_since(SystemTime::UNIX_EPOCH)
//             .expect("Failed to convert to duration")
//             .as_secs()
//             * 1000;
//
//         assert!(!is_date_older_than_n_seconds(five_seconds_ago, &10));
//     }
//
//     #[test]
//     fn edge_case_exact_n_seconds_old() {
//         // To ensure the test's robustness, consider the timing issue and test with a buffer
//         let slightly_more_than_ten_seconds_ago = SystemTime::now()
//             .checked_sub(Duration::from_secs(11))
//             .expect("Failed to calculate time")
//             .duration_since(SystemTime::UNIX_EPOCH)
//             .expect("Time went backwards")
//             .as_secs()
//             * 1000; // Convert to milliseconds
//
//         assert!(is_date_older_than_n_seconds(
//             slightly_more_than_ten_seconds_ago,
//             &10
//         ));
//     }
// }
