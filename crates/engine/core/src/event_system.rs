use std::sync::RwLock;

struct EventManager<T> {
    callbacks: RwLock<Vec<Box<dyn FnMut(&T) -> bool>>>
}

impl<T> Default for EventManager<T> {
    fn default() -> Self {
        Self { callbacks: Default::default() }
    }
}

impl<T> EventManager<T> {

    pub fn execute(&self, context: &T) {
        let callbacks = &mut *self.callbacks.write().unwrap();
        let mut removed_indices = vec![];
        for (i, callback) in callbacks.iter_mut().enumerate() {
            if !callback(context) {
                removed_indices.push(i);
            }
        }
        if !removed_indices.is_empty() {
            for i in removed_indices.len()..0 {
                let _dropped_cb = callbacks.remove(i);
            }
        }
    }

    pub fn bind<C: FnMut(&T) -> bool + 'static>(&self, callback: C) {
        self.callbacks.write().unwrap().push(Box::new(callback));
    }
}


#[cfg(test)]
mod events_tests {
    use crate::event_system::EventManager;

    #[test]
    fn test() {
        let event_manager = EventManager::<f32>::default();


        event_manager.bind(move |data| {
            assert_eq!(*data, 3.5);
            true
        });

        event_manager.execute(&3.5);
    }
}