pub mod actors;
pub mod config;
#[cfg(feature = "streaming")]
pub mod streaming;
#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        assert_eq!(2 + 2, 4);
    }
}