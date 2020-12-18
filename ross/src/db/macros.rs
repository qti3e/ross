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

        pub trait $cf_trait<K: serde::Serialize + serde::de::DeserializeOwned>: Sized {
            /// Returns the column family used to store this key.
            fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily;

            /// Returns a reference to the inner key.
            fn key(&self) -> &K;

            /// Returns an iterator over all of the keys in the given database, that are
            /// of the same type as Self and start with the given prefix.
            /// ```
            /// let db = DB::open("/tmp");
            /// let iter = keys::Branch::key_iterator(&db, &[]);
            /// for branch_id in iter { }
            /// ```
            #[inline]
            fn key_iterator<'d, P: AsRef<[u8]>>(db: &'d DB, prefix: P) -> KeyIterator<'d, K> {
                db.prefix_key_iterator::<'d, 'd, K, Self, P>(prefix)
            }
        }

        pub trait $read_trait<Key: serde::Serialize + serde::de::DeserializeOwned, Value: serde::Serialize + serde::de::DeserializeOwned>: $cf_trait<Key> {
            /// Returns an iterator over all of the key-value pairs in the given database that are
            /// of the same type as Self and their key starts with the given prefix.
            /// ```
            /// let db = DB::open("/tmp");
            /// let iter = keys::Branch::key_value_iterator(&db, &[]);
            /// for (branch_id, branch_info) in iter { }
            /// ```
            #[inline]
            fn key_value_iterator<'d, P: AsRef<[u8]>>(db: &'d DB, prefix: P) -> KeyValueIterator<'d, Key, Value> {
                db.prefix_iterator::<'d, 'd, Key, Value, Self, P>(prefix)
            }
        }

        pub trait $write_trait<Key: serde::Serialize + serde::de::DeserializeOwned, Value: serde::Serialize + serde::de::DeserializeOwned>: $cf_trait<Key> {}

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
            #[derive(Debug)]
            pub struct $key_name<'a>(pub &'a $key_type);

            impl<'a> $cf_trait<$key_type> for $key_name<'a> {
                #[inline]
                fn key(&self) -> &$key_type { self.0 }

                #[inline]
                fn cf<'c>(cf: &'c CF) -> &'c rocksdb::ColumnFamily {
                    &cf.$cf_name
                }
            }

            impl<'a> $read_trait<$key_type, $value_type> for $key_name<'a> {}
            impl<'a> $write_trait<$key_type, $value_type> for $key_name <'a>{}

            $(
                $(#[$partial_attr])*
                #[derive(Debug)]
                pub struct $partial_name<'a>(pub &'a $key_type);

                impl<'a> $cf_trait<$key_type> for $partial_name<'a> {
                    #[inline]
                    fn key(&self) -> &$key_type { self.0 }

                    #[inline]
                    fn cf<'c>(cf: &'c CF) -> &'c rocksdb::ColumnFamily {
                        &cf.$cf_name
                    }
                }

                impl<'a> $read_trait<$key_type, $partial_type> for $partial_name<'a> {}
            )*
        )*
    }
}
