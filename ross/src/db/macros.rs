#[macro_export]
macro_rules! db_schema {
    (
        $trait_name:ident / $read_trait_name:ident {
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

        pub trait $trait_name<Value> {
            fn serialize(self) -> Vec<u8>;
            fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily;
        }

        pub trait $read_trait_name<Value> {
            fn serialize(self) -> Vec<u8>;
            fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily;
        }

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

            impl $trait_name<$value_type> for $key_name {
                #[inline]
                fn serialize(self) -> Vec<u8> {
                    bincode::serialize(&self.0).unwrap()
                }

                #[inline]
                fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily {
                    &cf.$cf_name
                }
            }

            impl $read_trait_name<$value_type> for $key_name {
                #[inline]
                fn serialize(self) -> Vec<u8> {
                    bincode::serialize(&self.0).unwrap()
                }

                #[inline]
                fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily {
                    &cf.$cf_name
                }
            }

            $(
                $(#[$partial_attr])*
                #[derive(Debug, Serialize, Deserialize)]
                struct $partial_name(pub $key_type);

                impl $read_trait_name<$partial_type> for $partial_name {
                    #[inline]
                    fn serialize(self) -> Vec<u8> {
                        bincode::serialize(&self.0).unwrap()
                    }

                    #[inline]
                    fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily {
                        &cf.$cf_name
                    }
                }
            )*
        )*
    }
}
