pub use anyhow::Result;

#[macro_export]
macro_rules! generate_tests {
  ($runner: ident; $package: ident; $( $func: ident ),*) => {
    $(
    #[tokio::test]
    async fn $func() -> ::test_suite::Result<()> {
      ::test_suite::$package::$func($runner).await
    }
    )*
  };
}
