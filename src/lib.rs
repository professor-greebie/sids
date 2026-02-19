pub mod actors;
#[cfg(feature = "streaming")]
pub mod streaming;
#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        assert_eq!(2 + 2, 4);
    }
}