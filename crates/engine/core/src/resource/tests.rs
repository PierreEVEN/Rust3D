
#[cfg(test)]
mod resources_tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64};
    use std::sync::atomic::Ordering::SeqCst;
    use logger::{error};
    use crate::engine::{Engine};
    use crate::resource::resource::{Resource, ResourceFactory, ResourceStorage};

    struct TestResource {
        pub value: i64,
        allocation_status: Arc<AtomicI64>
    }

    impl Resource for TestResource {
        fn type_name(&self) -> &str {
            "Custom resource type"
        }

        fn name(&self) -> &str {
            "Custom resource name"
        }
    }

    impl Drop for TestResource {
        fn drop(&mut self) {
            self.allocation_status.fetch_sub(1, SeqCst);
        }
    }

    struct TestResourceFactory {
        allocation_status: Arc<AtomicI64>,
        value: i64,
    }

    impl ResourceFactory<TestResource> for TestResourceFactory {
        fn instantiate(self) -> TestResource {
            self.allocation_status.fetch_add(1, SeqCst);
            TestResource { value: self.value, allocation_status: self.allocation_status.clone() }
        }
    }

    #[test]
    fn create() {
        crate::test_tools::engine_test_tools::run_with_test_engine(|| {
            for _i in 0..10000 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let storage = ResourceStorage::default();
                storage.load(TestResourceFactory { value: 12, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 12);
                assert_eq!(allocation_status.load(SeqCst), 1);
            }
        });
    }

    #[test]
    fn destroy() {
        crate::test_tools::engine_test_tools::run_with_test_engine(|| {
            for _i in 0..10000 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let storage = ResourceStorage::default();
                storage.load(TestResourceFactory { value: 12, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 12);
                storage.destroy();
                Engine::get().resource_allocator().flush();
                assert_eq!(allocation_status.load(SeqCst), 0);
            }
        });
    }

    #[test]
    fn replace() {
        crate::test_tools::engine_test_tools::run_with_test_engine(|| {
            for _i in 0..10000 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let storage = ResourceStorage::default();
                storage.load(TestResourceFactory { value: 12, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 12);
                storage.destroy();
                storage.load(TestResourceFactory { value: 50, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 50);
                storage.load(TestResourceFactory { value: 25, allocation_status: allocation_status.clone() });
                assert_eq!(storage.unwrap().value, 25);
                storage.destroy();
                Engine::get().resource_allocator().flush();
                assert_eq!(allocation_status.load(SeqCst), 0);
            }
        });
    }

    #[test]
    fn accumulate() {
        crate::test_tools::engine_test_tools::run_with_test_engine(|| {
            for _i in 0..100 {
                let allocation_status = Arc::new(AtomicI64::new(0));
                let mut storages = vec![];
                for j in 0..100 {
                    {
                        let storage = ResourceStorage::default();
                        storage.load(TestResourceFactory { value: j, allocation_status: allocation_status.clone() });
                        assert!(storage.is_valid());
                        storages.push(storage);
                        assert!(storages.last().unwrap().is_valid());
                    }
                    assert!(storages.last().unwrap().is_valid());
                }
                assert_eq!(allocation_status.load(SeqCst), 100);

                for j in 0..100 {
                    if !storages[j].is_valid() {
                        error!("storage[{j}] is not valid");
                    }
                    assert!(storages[j].is_valid());
                    assert_eq!(storages[j].unwrap().value, j as i64);
                    storages[j].destroy();
                }
                Engine::get().resource_allocator().flush();
                assert_eq!(allocation_status.load(SeqCst), 0);
            }
        });
    }
}