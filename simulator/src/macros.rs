#[macro_export]
macro_rules! boxed_node {
    ( $x:ty ) => {
        Box::new(<$x>::new())
    };
}
