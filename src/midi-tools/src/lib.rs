#![feature(generators)]
#![feature(associated_type_bounds)]

pub mod events;
pub mod io;
pub mod notes;
pub mod num;
pub mod sequence;

#[macro_export]
macro_rules! pipe {
    ($var:tt |> $function: ident($($params: expr),*) $($calls:tt)*) => {
        pipe!({$function($var, $($params),*)} $($calls)*)
    };
    ($var:tt |> $namespace1:ident :: $function: ident($($params: expr),*) $($calls:tt)*) => {
        pipe!({$namespace1::$function($var, $($params),*)} $($calls)*)
    };
    ($var:tt |> $namespace1:ident :: $namespace2:ident :: $function: ident($($params: expr),*) $($calls:tt)*) => {
        pipe!({$namespace1::$namespace2::$function($var, $($params),*)} $($calls)*)
    };
    ($var:tt |> $namespace1:ident :: $namespace2:ident :: $namespace3:ident :: $function: ident($($params: expr),*) $($calls:tt)*) => {
        pipe!({$namespace1::$namespace2::$namespace3::$function($var, $($params),*)} $($calls)*)
    };
    ($var:tt |> $namespace1:ident :: $namespace2:ident :: $namespace3:ident :: $namespace4:ident :: $function: ident($($params: expr),*) $($calls:tt)*) => {
        pipe!({$namespace1::$namespace2::$namespace3::$namespace4::$function($var, $($params),*)} $($calls)*)
    };
    ($var:tt . $function: ident $( :: < $($types: tt $(< $types2: tt >)? ),* > )? ( $($params: expr),* ) $($calls:tt)*) => {
        pipe!({$var.$function $( :: < $($types $(< $types2 >)?),* > )? ( $($params),* )} $($calls)*)
    };
    ($var:tt) => {
        $var
    };
}

#[macro_export]
macro_rules! unwrap {
    ($val:expr) => {
        match $val {
            Ok(v) => v,
            Err(e) => {
                yield Err(e);
                panic!("Iterator requested the next item after an error occured");
            }
        }
    };
}
