fn cpu() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu() {
        assert_eq!(cpu(), ());
    }
}
