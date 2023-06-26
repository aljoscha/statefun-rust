/// This macro allows passing in a variadic list of ValueSpec<T>'s without having to cast them
/// to their base type. Use this macro in the call to `register_fn()` to simplify your code.
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
