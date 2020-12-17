#[macro_export]
macro_rules! db_schema {
    (
        ($cf_trait:ident, $write_trait:ident, $read_trait:ident) {
            $(
                $(#[$attr:meta])*
                cf $cf_name:ident($key_name:ident:$key_type:ty) -> $value_type:ty {
                    $(
                        $(#[$partial_attr:meta])*
                        $partial_name:ident -> $partial_type:ty;
                    )*
                }
            ),*
        }
    ) => {
        $(
            pub(super) const $cf_name: &str = stringify!($cf_name);
        )*

        pub trait $cf_trait {
            fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily;
        }

        pub trait $read_trait<Value>: $cf_trait {}
        pub trait $write_trait<Value>: $cf_trait {}

        #[allow(non_snake_case)]
        pub struct CF {
            $(
                $(#[$attr])*
                $cf_name: rocksdb::ColumnFamily
            ),*
        }

        impl CF {
            pub(super) fn new(db: &rocksdb::DB) -> Self {
                unsafe {
                    Self {
                        $(
                            $cf_name: std::mem::transmute_copy(db.cf_handle($cf_name).unwrap())
                        ),*
                    }
                }
            }
        }

        $(
            $(#[$attr])*
            #[derive(Debug, Serialize, Deserialize)]
            struct $key_name(pub $key_type);

            impl $cf_trait for $key_name {
                #[inline]
                fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily {
                    &cf.$cf_name
                }
            }

            impl $read_trait<$value_type> for $key_name {}
            impl $write_trait<$value_type> for $key_name {}

            $(
                $(#[$partial_attr])*
                #[derive(Debug, Serialize, Deserialize)]
                struct $partial_name(pub $key_type);

                impl $cf_trait for $partial_name {
                    #[inline]
                    fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily {
                        &cf.$cf_name
                    }
                }

                impl $read_trait<$partial_type> for $partial_name {}
            )*
        )*
    }
}
