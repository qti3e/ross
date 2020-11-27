#[macro_export]
macro_rules! sync {
    (
        $(#[$attr:meta])*
        $n:ident ($inner:ty) {
            $($(#[$field_attr:meta])* $name:ident : $type:ty),*
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone)]
        pub struct $n {
            inner: std::sync::Arc<crossbeam::sync::ShardedLock<$inner>>,
            $($(#[$field_attr])* $name : $type),*
        }

        impl $n {
            pub fn new(
                inner: $inner,
                $($name : $type),*
            ) -> Self {
                Self {
                    inner: std::sync::Arc::new(crossbeam::sync::ShardedLock::new(inner)),
                    $($name),*
                }
            }

            pub fn read(
                &self,
            ) -> crate::error::Result<crossbeam::sync::ShardedLockReadGuard<$inner>> {
                self.inner
                    .read()
                    .map_err(|_| crate::error::Error::AcquireReadLock)
            }

            pub fn write(
                &self,
            ) -> crate::error::Result<crossbeam::sync::ShardedLockWriteGuard<$inner>> {
                self.inner
                    .write()
                    .map_err(|_| crate::error::Error::AcquireWriteLock)
            }
        }
    };
}

#[macro_export]
macro_rules! db_keys {
    (
        $trait_name:ident($name:ident) {
            $(
                $(#[$attr:meta])*
                $key_name:ident($key_type:ty) -> $value_type:ty
            ),*
        }
    ) => {
        pub trait $trait_name<Value> {
            fn key(self) -> $name;
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub enum $name {
            $($key_name($key_type)),*
        }

        $(
            $(#[$attr])*
            #[derive(Debug, Serialize, Deserialize)]
            pub struct $key_name(pub $key_type);

            impl $trait_name<$value_type> for $key_name {
                fn key(self) -> $name {
                    $name::$key_name(self.0)
                }
            }
        )*
    }
}

#[macro_export]
macro_rules! db_partial_keys {
    (
        $trait_name:ident($name:ident)::$partial_trait_name:ident {
            $(
                $(#[$attr:meta])*
                $key_name:ident($key_type:ty)::$partial_key_name:ident -> $value_type:ty
            ),*
        }
    ) => {
        pub trait $partial_trait_name<Value> {
            fn key(self) -> $name;
        }

        $(
            $(#[$attr])*
            #[derive(Debug, Serialize, Deserialize)]
            pub struct $partial_key_name(pub $key_type);

            impl $partial_trait_name<$value_type> for $partial_key_name {
                fn key(self) -> $name {
                    $name::$key_name(self.0)
                }
            }
        )*
    }
}

#[macro_export]
macro_rules! options {
    (
        $(#[$options_attr:meta])*
        $builder_name:ident($options_name:ident) {
            $($(#[$attr:meta])* $name:ident : $type:ty = $default:expr),*
        }) => {
            $(#[$options_attr:meta])*
            pub struct $options_name {
                $($(#[$attr])* pub $name : $type),*
            }

            pub struct $builder_name {
                $($(#[$attr])* $name : Option<$type>),*
            }

            impl $builder_name {
                $(
                    $(#[$attr])*
                    pub fn $name<T: Into<$type>>(mut self, data: T) -> Self {
                        self.$name = Some(data.into());
                        self
                    }
                )*

                /// Finalize and build the options.
                pub fn build(self) -> $options_name {
                    $options_name {
                        $($name: self.$name.expect("Required option was not provided.")),*
                    }
                }
            }

            impl Default for $builder_name {
                fn default() -> Self {
                    Self {
                        $($name: $default),*
                    }
                }
            }
        };
}
