use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct LRUCache<K, V> where K: Clone + Eq + Hash {
    capacity: usize,
    cache: Arc<Mutex<HashMap<K, (V, Instant)>>>,
    order: Arc<Mutex<VecDeque<K>>>,
}

impl<K: Eq + Hash + Clone + Default + Send + 'static, V: Default + Send + 'static> LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        let cache = Arc::new(Mutex::new({
            let mut map = HashMap::with_capacity(capacity);
            // Use dummy values for type inference
            map.insert(K::default(), (V::default(), Instant::now()));
            map.clear();
            map
        }));
        let order = Arc::new(Mutex::new(VecDeque::with_capacity(capacity)));

        let cache_clone = Arc::clone(&cache);
        let order_clone = Arc::clone(&order);

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(30));
                let mut cache = cache_clone.lock().unwrap();
                let mut order = order_clone.lock().unwrap();

                let keys_to_remove: Vec<K> = cache.iter()
                    .filter(|(key, (_, creation_time))| Instant::now().duration_since(*creation_time) > Duration::from_secs(30))
                    .map(|(key, _)| key.clone())
                    .collect();

                for key in keys_to_remove {
                    cache.remove(&key);
                    order.retain(|k| k != &key);
                }
            }
        });

        LRUCache {
            capacity,
            cache,
            order,
        }
    }

    fn get(&mut self, key: &K) -> Option<V> where V: Clone {
        if let Some((value, _)) = self.cache.lock().unwrap().get(key) {
            self.order.lock().unwrap().retain(|k| k != k);
            self.order.lock().unwrap().push_front(key.clone());

            Some(value.clone())
        } else {
            None
        }
    }

    fn put(&mut self, key: K, val: V, time_based: bool) {
        if self.cache.lock().unwrap().len() >= self.capacity {
            if let Some(oldest_key) = self.order.lock().unwrap().pop_back() {
                self.cache.lock().unwrap().remove(&oldest_key);
            }
        }

        let entry = if time_based {
            (val, Instant::now())
        } else {
            (val, Instant::now() + Duration::from_secs(10))
        };

        self.cache.lock().unwrap().insert(key.clone(), entry);
        self.order.lock().unwrap().push_front(key);
    }

    fn is_expired(&self, key: &K, max_age: Duration) -> bool {
        if let Some((_, creation_time)) = self.cache.lock().unwrap().get(key) {
            Instant::now().duration_since(*creation_time) > max_age
        } else {
            false
        }
    }
}
fn main() {
    let mut crow_cache = LRUCache::new(3);

    // Insert some values (time-based and non-time-based)
    crow_cache.put("twig", 1, false);
    crow_cache.put("egg", 2, true);
    crow_cache.put("branch", 3, false);

    // Retrieve values
    println!("{:?}", crow_cache.get(&"branch").unwrap());
    println!("{:?}", crow_cache.get(&"egg").unwrap());
    println!("{:?}", crow_cache.get(&"twig").unwrap());

    let max_age = Duration::from_secs(10);
    std::thread::sleep(max_age);
    if crow_cache.is_expired(&"egg", max_age) {
        println!("the egg has hatched and left the nest!!!");
    }
    println!("{:?}", crow_cache.get(&"egg").unwrap_or(0));
}
