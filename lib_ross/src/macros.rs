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
