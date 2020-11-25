#[macro_export]
macro_rules! sync {
    (
        sync $n:ident ($inner:ty) {
            $($name:ident : $type:ty),*
        }
    ) => {
        #[derive(Clone)]
        pub struct $n {
            inner: std::sync::Arc<crossbeam::sync::ShardedLock<$inner>>,
            $($name : $type),*
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
            $($key_name($key_name)),*
        }

        $(
            $(#[$attr])*
            #[derive(Debug, Serialize, Deserialize)]
            pub struct $key_name(pub $key_type);

            impl $trait_name<$value_type> for $key_name {
                fn key(self) -> $name {
                    $name::$key_name(self)
                }
            }
        )*
    }
}
