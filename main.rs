use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const N: usize = 4; 
const MEALS: usize = 5; 
const THINK_TIME: Duration = Duration::from_millis(100);
const EAT_TIME: Duration = Duration::from_millis(150);

struct Table {
    queue: VecDeque<usize>,
    eating: [bool; N],
}

type Shared = Arc<(Mutex<Table>, Condvar)>;

#[derive(Clone, Copy, Default)]
struct Stats {
    meals: u64,
    total_wait: Duration,
    max_wait: Duration,
}

struct Philosopher {
    id: usize,
    left: usize,  
    right: usize, 
}

impl Philosopher {
    fn new(id: usize) -> Self {
        Philosopher {
            id,
            left: id,
            right: (id + 1) % N,
        }
    }

    fn run(&self, shared: &Shared) -> Stats {
        let (lock, cv) = &**shared;
        let left_neighbor = (self.id + N - 1) % N;
        let right_neighbor = (self.id + 1) % N;
        let mut stats = Stats::default();

        for meal in 1..=MEALS {
            println!("Philosopher {} is thinking", self.id);
            thread::sleep(THINK_TIME);

            println!("Philosopher {} is hungry", self.id);
            let start = Instant::now();

            
            let mut t = lock.lock().unwrap();
            t.queue.push_back(self.id);
            loop {
                let neighbor_eating = t.eating[left_neighbor] || t.eating[right_neighbor];
                let neighbor_ahead = t
                    .queue
                    .iter()
                    .take_while(|&&x| x != self.id)
                    .any(|&x| x == left_neighbor || x == right_neighbor);
                if !neighbor_eating && !neighbor_ahead {
                    break;
                }
                t = cv.wait(t).unwrap();
            }
            t.queue.retain(|&x| x != self.id);
            t.eating[self.id] = true;
            drop(t);

            let waited = start.elapsed();
            stats.meals += 1;
            stats.total_wait += waited;
            if waited > stats.max_wait {
                stats.max_wait = waited;
            }

            println!("Philosopher {} picked up chopstick {}", self.id, self.left);
            println!("Philosopher {} picked up chopstick {}", self.id, self.right);
            println!("Philosopher {} is EATING (meal {}/{})", self.id, meal, MEALS);
            thread::sleep(EAT_TIME);

            let mut t = lock.lock().unwrap();
            t.eating[self.id] = false;
            cv.notify_all();
            drop(t);

            println!(
                "Philosopher {} finished, putting down chopsticks {} and {}",
                self.id, self.left, self.right
            );
        }
        println!("Philosopher {} is done", self.id);
        stats
    }
}

fn report(stats: &[Stats]) {
    println!("\n=== FIFO-fair waiter ===");
    println!(
        "{:<6} {:>8} {:>16} {:>16}",
        "phil", "meals", "avg wait (ms)", "max wait (ms)"
    );
    for (i, s) in stats.iter().enumerate() {
        let avg_ms = if s.meals > 0 {
            s.total_wait.as_secs_f64() * 1000.0 / s.meals as f64
        } else {
            0.0
        };
        println!(
            "{:<6} {:>8} {:>16.2} {:>16.2}",
            i,
            s.meals,
            avg_ms,
            s.max_wait.as_secs_f64() * 1000.0
        );
    }
    let min = stats.iter().map(|s| s.meals).min().unwrap_or(0);
    let max = stats.iter().map(|s| s.meals).max().unwrap_or(0);
    let ratio = if max > 0 { min as f64 / max as f64 } else { 0.0 };
    println!(
        "fairness (min/max meals): {:.3}  (1.0 = perfectly even)",
        ratio
    );
}

fn main() {
    let shared: Shared = Arc::new((
        Mutex::new(Table {
            queue: VecDeque::new(),
            eating: [false; N],
        }),
        Condvar::new(),
    ));

    let handles: Vec<_> = (0..N)
        .map(|id| {
            let shared = Arc::clone(&shared);
            let philosopher = Philosopher::new(id);
            thread::spawn(move || philosopher.run(&shared))
        })
        .collect();

    let stats: Vec<Stats> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    println!("\nAll philosophers have finished dining.");
    report(&stats);
}
