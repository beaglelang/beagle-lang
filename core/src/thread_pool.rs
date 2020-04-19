use std::thread;
use std::thread::{
    JoinHandle
};
use std::sync::{
    Arc,
    Mutex,
    Condvar,
};
use std::collections::VecDeque;

pub type Job<'a> = dyn FnOnce() -> Result<(), ()> + Send + Sync + 'a;

// struct ThreadPoolState {
//     inner: Mutex<(VecDeque<Box<Job>>, bool)>,
//     on_change: Condvar,
// }

// pub struct ThreadPool {
//     threads: Vec<JoinHandle<()>>,
//     state: Arc<ThreadPoolState>,
// }

// impl ThreadPool {
//     pub fn new(num_threads: u8) -> Self {
//         let mut threads = Vec::with_capacity(num_threads as usize);

//         let state = Arc::new(ThreadPoolState {
//             inner: Mutex::new((VecDeque::new(), false)),
//             on_change: Condvar::new(),
//         });

//         for _ in 0..num_threads {
//             let pool_state = state.clone();

//             let handle = thread::spawn(move || {
//                 loop {     
//                     let mut guard = pool_state.inner.lock().unwrap();
//                     let (jobs, is_stopping) = &mut *guard;
                    
//                     if *is_stopping { break; }
                    
//                     let job = if !jobs.is_empty() {
//                         jobs.pop_front()
//                     } else {
//                         let mut guard = pool_state.on_change.wait(guard).unwrap();

//                         let (jobs, is_stopping) = &mut *guard;

//                         if *is_stopping { break; }
                        
//                         jobs.pop_front()
//                     }.unwrap();

//                     job();
//                 }
//             });

//             threads.push(handle);
//         }

//         Self {
//             threads,
//             state,
//         }
//     }

//     pub fn enqueue<F, T>(&mut self, f: F) -> Future<T>
//     where
//         F: FnOnce() -> T,
//         F: Send + Sync + 'static,
//         T: Send + Sync + 'static    
//     {
//         let inner = &self.state.inner;
//         let mut guard = inner.lock().unwrap();

//         let (jobs, _) = &mut *guard;
        
//         let my_packet : Arc<(Mutex<Option<T>>, Condvar)> = Arc::new((Mutex::new(None), Condvar::new()));
//         let their_packet = my_packet.clone();

//         jobs.push_back(Box::new(move || {
//             let (inner, on_change) = &*their_packet;
//             {
//                 let mut guard = inner.lock().unwrap();
//                 *guard = Some(f());
//             }
//             on_change.notify_all();
//         }));

//         self.state.on_change.notify_one();

//         Future {
//             inner: my_packet,
//         }
//     }
// }

// impl Drop for ThreadPool {
//     fn drop(&mut self) {
//         {
//             let lock = &self.state.inner;
//             let mut guard = lock.lock().unwrap();
        
//             let (_, is_stopping) = &mut *guard;
//             *is_stopping = true;
//         }
//         self.state.on_change.notify_all();

//         self.threads.drain(..).for_each(|thread| thread.join().unwrap());
//     }
// }

// pub struct Future<T> {
//     inner: Arc<(Mutex<Option<T>>, Condvar)>,
// }

// impl<T> Future<T> {
//     pub fn get(&self) -> T {
//         let (inner, on_change) = &*self.inner;
//         let mut guard = inner.lock().unwrap();
        
//         if guard.is_some() {
//             return guard.take().unwrap();
//         }

//         let mut guard = on_change.wait(guard).unwrap();
//         guard.take().unwrap()
//     }
// }