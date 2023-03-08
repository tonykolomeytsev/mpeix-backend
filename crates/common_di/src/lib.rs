#[macro_export]
macro_rules! di_constructor {
    ($struct_name:tt { $( $dep_name:ident : $dep_type:ty ),* }) => {
        impl $struct_name {
            pub fn new( $( $dep_name : $dep_type ),* ) -> $struct_name {
                $struct_name { $( $dep_name ),* }
            }
        }
    };
    ($struct_name:tt ( $( $dep_name:ident : $dep_type:ty ),* )) => {
        impl $struct_name {
            pub fn new( $( $dep_name : $dep_type ),* ) -> $struct_name {
                $struct_name ( $( $dep_name ),* )
            }
        }
    };
}
