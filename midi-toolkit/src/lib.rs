#![feature(generators)]
#![feature(associated_type_bounds)]

use crossbeam_channel::{Receiver, RecvError};

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
    ($var:tt |> $namespace1:ident :: < $($types: tt ),+ > :: $function: ident($($params: expr),*) $($calls:tt)*) => {
        pipe!({$namespace1::<$($types,)+>::$function($var, $($params),*)} $($calls)*)
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
    ($var:tt . $field: ident $($calls:tt)* ) => {
        pipe!({ $var.$field } $($calls)*)
    };
    ($var:tt) => {
        $var
    };
}

#[macro_export(local_inner_macros)]
macro_rules! unwrap {
    ($val:expr) => {
        match $val {
            Ok(v) => v,
            Err(e) => {
                yield_error!(Err(e));
            }
        }
    };
}

#[macro_export]
macro_rules! yield_error {
    ($err:expr) => {{
        yield $err;
        panic!("Iterator requested the next item after an error occured");
    }};
}

pub struct DelayedReceiver<T> {
    first: Option<Result<T, RecvError>>,
    receiver: Receiver<T>,
}

impl<T> DelayedReceiver<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        Self {
            first: None,
            receiver,
        }
    }

    pub fn wait_first(&mut self) {
        if self.first.is_none() {
            self.first = Some(self.receiver.recv());
        }
    }

    pub fn recv(&mut self) -> Result<T, RecvError> {
        if let Some(val) = self.first.take() {
            val
        } else {
            self.receiver.recv()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delayed_receiver() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut receiver = DelayedReceiver::new(receiver);
        {
            let sender = sender;
            sender.send(1).unwrap();
            sender.send(2).unwrap();
            sender.send(3).unwrap();
            sender.send(4).unwrap();
        }
        receiver.wait_first();
        assert_eq!(receiver.recv().unwrap(), 1);
        assert_eq!(receiver.recv().unwrap(), 2);
        assert_eq!(receiver.recv().unwrap(), 3);
        assert_eq!(receiver.recv().unwrap(), 4);
    }
}
