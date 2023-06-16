///
#[macro_export]
macro_rules! specs {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.into());
            )*
            temp_vec
        }
    };
}
