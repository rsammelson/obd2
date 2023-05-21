macro_rules! trait_func {
    {
        $(#[$attr:meta])*
        fn $name:ident($service:expr) -> $retrieve_type:ty;
    } => {
        $(#[$attr])*
        fn $name(&mut self) -> Result<Vec<$retrieve_type>>;
    };
    {
        $(#[$attr:meta])*
        fn $name:ident($service:expr, $pid:expr) -> $retrieve_type:ty;
    } => {
        $(#[$attr])*
        fn $name(&mut self) -> Result<Vec<$retrieve_type>>;
    };
    {
        $(#[$attr:meta])*
        fn $name:ident($service:expr, $pid:expr, $map:expr) -> $retrieve_type:ty;
    } => {
        $(#[$attr])*
        fn $name(&mut self) -> Result<Vec<$retrieve_type>>;
    };
    {
        $(#[$attr:meta])*
        fn $name:ident<$retrieve_type:ty>($service:expr, $pid:expr, $map:expr) -> $out_type:ty;
    } => {
        $(#[$attr])*
        fn $name(&mut self) -> Result<Vec<$out_type>>;
    };
}

macro_rules! impl_func {
    {
        $(#[$attr:meta])*
        fn $name:ident($service:expr) -> $retrieve_type:ty;
    } => {
        fn $name(&mut self) -> Result<Vec<$retrieve_type>> {
            <$retrieve_type>::get_obd2_val_mode(self, $service)
        }
    };
    {
        $(#[$attr:meta])*
        fn $name:ident($service:expr, $pid:expr) -> $retrieve_type:ty;
    } => {
        fn $name(&mut self) -> Result<Vec<$retrieve_type>> {
            <$retrieve_type>::get_obd2_val(self, $service, $pid)
        }
    };
    {
        $(#[$attr:meta])*
        fn $name:ident($service:expr, $pid:expr, $map:expr) -> $retrieve_type:ty;
    } => {
        fn $name(&mut self) -> Result<Vec<$retrieve_type>> {
            $map(<$retrieve_type>::get_obd2_val(self, $service, $pid))
        }
    };
    {
        $(#[$attr:meta])*
        fn $name:ident<$retrieve_type:ty>($service:expr, $pid:expr, $map:expr) -> $out_type:ty;
    } => {
        fn $name(&mut self) -> Result<Vec<$out_type>> {
            Ok(
                <$retrieve_type>::get_obd2_val(self, $service, $pid)?
                    .into_iter()
                    .map(|v| $map(v.into()))
                    .collect()
            )
        }
    };
}

macro_rules! func {
    {
        $(#[$attr:meta])*
        trait $trait_name:ident;

        $({
            $(
                $(#[$f_attr_inner:meta])*
                fn $f_name:ident($self:ident, $f_service:expr$(, $f_pid:expr)?) -> $f_output:ty
                    $inside:block
            )+
        })?

        $(
            $(#[$attr_inner:meta])*
            fn $name:ident$(<$retrieve_type:ty>)?($service:expr$(, $pid:expr$(, $map:expr)?)?) -> $output:ty;
         )*
    } => {
        $(#[$attr])*
        pub trait $trait_name: private::Sealed {
            $($(
                $(#[$f_attr_inner])*
                fn $f_name(&mut self) -> $f_output;
            )+)?

            $(
                trait_func! {
                    $(#[$attr_inner])*
                    ///
                    #[doc=concat!(
                        "Details: service ", $service,
                        $(", PID ", $pid,)?
                        ", read type: `", decode_type!($output $(, $retrieve_type)?), "`"
                    )]
                    fn $name$(<$retrieve_type>)?($service$(, $pid$(, $map)?)?) -> $output;
                }
            )*
        }

        impl<T: Obd2Device> $trait_name for T {
            $($(
                fn $f_name(&mut $self) -> $f_output
                    $inside
            )+)?

            $(
                impl_func! {
                    $(#[$attr_inner])*
                    fn $name$(<$retrieve_type>)?($service$(, $pid$(, $map)?)?) -> $output;
                }
            )*
        }
    };
}

macro_rules! decode_type {
    ($_:ty, $t:ty) => {
        stringify!($t)
    };
    ($t:ty) => {
        stringify!($t)
    };
}
