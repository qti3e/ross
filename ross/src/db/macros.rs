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

        pub trait $cf_trait: Sized {
            /// Actual type of the key.
            type Key: serde::Serialize + serde::de::DeserializeOwned;

            /// Returns the column family used to store this key.
            fn cf<'a>(cf: &'a CF) -> &'a rocksdb::ColumnFamily;

            /// Returns a reference to the inner key.
            fn key(&self) -> &Self::Key;

            /// Returns an iterator over all of the keys in the given database, that are
            /// of the same type as Self and start with the given prefix.
            /// ```
            /// let db = DB::open("/tmp");
            /// let iter = keys::Branch::key_iterator(&db, &[]);
            /// for branch_id in iter { }
            /// ```
            #[inline]
            fn key_iterator<'d: 'b, 'b, P: serde::Serialize>(db: &'d DB, prefix: &P) -> KeyIterator<'b, Self::Key> {
                db.prefix_key_iterator::<'d, 'b, Self, Vec<u8>>(serialize(prefix))
            }
        }

        pub trait $read_trait: $cf_trait {
            /// Type of the value associated with this key.
            type Value: serde::Serialize + serde::de::DeserializeOwned;

            /// Returns an iterator over all of the key-value pairs in the given database that are
            /// of the same type as Self and their key starts with the given prefix.
            /// ```
            /// let db = DB::open("/tmp");
            /// let iter = keys::Branch::key_value_iterator(&db, &[]);
            /// for (branch_id, branch_info) in iter { }
            /// ```
            #[inline]
            fn key_value_iterator<'d: 'b, 'b, P: serde::Serialize>(db: &'d DB, prefix: &P) -> KeyValueIterator<'b, Self::Key, Self::Value> {
                db.prefix_iterator::<'d, 'b, Self, Vec<u8>>(serialize(prefix))
            }
        }

        pub trait $write_trait: $read_trait {}

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

            impl<'a> $cf_trait for $key_name<'a> {
                type Key = $key_type;

                #[inline]
                fn key(&self) -> &Self::Key { self.0 }

                #[inline]
                fn cf<'c>(cf: &'c CF) -> &'c rocksdb::ColumnFamily {
                    &cf.$cf_name
                }
            }

            impl<'a> $read_trait for $key_name<'a> {
                type Value = $value_type;
            }

            impl<'a> $write_trait for $key_name<'a> {}

            $(
                $(#[$partial_attr])*
                #[derive(Debug)]
                pub struct $partial_name<'a>(pub &'a $key_type);

                impl<'a> $cf_trait for $partial_name<'a> {
                    type Key = $key_type;

                    #[inline]
                    fn key(&self) -> &Self::Key { self.0 }

                    #[inline]
                    fn cf<'c>(cf: &'c CF) -> &'c rocksdb::ColumnFamily {
                        &cf.$cf_name
                    }
                }

                impl<'a> $read_trait for $partial_name<'a> {
                    type Value = $partial_type;
                }
            )*
        )*
    }
}
