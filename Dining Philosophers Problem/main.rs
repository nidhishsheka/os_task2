use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const N: usize = 4; 
const MEALS: usize = 5; 
const THINK_TIME: Duration = Duration::from_millis(100);
const EAT_TIME: Duration = Duration::from_millis(150);
const ADDR: &str = "127.0.0.1:7878";
const PAGE: &str = include_str!("viz.html");



struct Inner {
    events: Vec<String>, 
    started: bool,
    t0: Instant,
}

struct Hub {
    inner: Mutex<Inner>,
    cv: Condvar,
}

impl Hub {
    fn new() -> Self {
        Hub {
            inner: Mutex::new(Inner {
                events: Vec::new(),
                started: false,
                t0: Instant::now(),
            }),
            cv: Condvar::new(),
        }
    }

    fn emit(&self, kind: &str, phil: usize, meal: usize, wait_ms: f64) {
        let mut g = self.inner.lock().unwrap();
        let t = g.t0.elapsed().as_millis();
        g.events.push(format!(
            r#"{{"t":{},"e":"{}","p":{},"m":{},"w":{:.2}}}"#,
            t, kind, phil, meal, wait_ms
        ));
        self.cv.notify_all();
    }

    fn start(&self) {
        let mut g = self.inner.lock().unwrap();
        if !g.started {
            g.started = true;
            g.t0 = Instant::now();
            self.cv.notify_all();
        }
    }

    fn wait_started(&self) {
        let mut g = self.inner.lock().unwrap();
        while !g.started {
            g = self.cv.wait(g).unwrap();
        }
    }
}


fn serve(listener: TcpListener, hub: Arc<Hub>) {
    for stream in listener.incoming().flatten() {
        let hub = Arc::clone(&hub);
        thread::spawn(move || handle(stream, hub));
    }
}

fn handle(mut s: TcpStream, hub: Arc<Hub>) {
    let mut buf = [0u8; 2048];
    let n = s.read(&mut buf).unwrap_or(0);
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();

    if path.starts_with("/events") {
        stream_events(s, hub);
    } else if path == "/" || path.starts_with("/?") {
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            PAGE.len()
        );
        let _ = s.write_all(head.as_bytes());
        let _ = s.write_all(PAGE.as_bytes());
    } else {
        let _ = s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    }
}

fn stream_events(mut s: TcpStream, hub: Arc<Hub>) {
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
    if s.write_all(head.as_bytes()).is_err() {
        return;
    }
    hub.start(); 

    let mut idx = 0;
    loop {
        let batch: Vec<String> = {
            let mut g = hub.inner.lock().unwrap();
            if idx >= g.events.len() {
                let (g2, _) = hub.cv.wait_timeout(g, Duration::from_secs(15)).unwrap();
                g = g2;
            }
            let b = g.events[idx..].to_vec();
            idx += b.len();
            b
        };
        let mut out = String::new();
        if batch.is_empty() {
            out.push_str(": keepalive\n\n");
        }
        for e in &batch {
            out.push_str(&format!("data: {}\n\n", e));
        }
        if s.write_all(out.as_bytes()).is_err() || s.flush().is_err() {
            return; 
        }
    }
}

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

    fn run(&self, shared: &Shared, hub: &Hub) -> Stats {
        let (lock, cv) = &**shared;
        let left_neighbor = (self.id + N - 1) % N;
        let right_neighbor = (self.id + 1) % N;
        let mut stats = Stats::default();

        for meal in 1..=MEALS {
            println!("Philosopher {} is thinking", self.id);
            hub.emit("think", self.id, meal, 0.0);
            thread::sleep(THINK_TIME);

            println!("Philosopher {} is hungry", self.id);
            let start = Instant::now();

            let mut t = lock.lock().unwrap();
            t.queue.push_back(self.id);
            hub.emit("hungry", self.id, meal, 0.0);
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
            let waited = start.elapsed();
            t.queue.retain(|&x| x != self.id);
            t.eating[self.id] = true;
            hub.emit("eat", self.id, meal, waited.as_secs_f64() * 1000.0);
            drop(t);

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
            hub.emit("put", self.id, meal, 0.0);
            cv.notify_all();
            drop(t);

            println!(
                "Philosopher {} finished, putting down chopsticks {} and {}",
                self.id, self.left, self.right
            );
        }
        println!("Philosopher {} is done", self.id);
        hub.emit("done", self.id, MEALS, 0.0);
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
    let listener = TcpListener::bind(ADDR)
        .unwrap_or_else(|e| panic!("cannot bind {} ({}). Is another copy still running?", ADDR, e));
    let hub = Arc::new(Hub::new());
    {
        let hub = Arc::clone(&hub);
        thread::spawn(move || serve(listener, hub));
    }

    println!("Open http://{} in your browser to start the simulation...", ADDR);
    hub.wait_started();

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
            let hub = Arc::clone(&hub);
            let philosopher = Philosopher::new(id);
            thread::spawn(move || philosopher.run(&shared, &hub))
        })
        .collect();

    let stats: Vec<Stats> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    println!("\nAll philosophers have finished dining.");
    report(&stats);
    hub.emit("end", 0, 0, 0.0);

    println!("\nServer still running (reload the page to replay). Press Ctrl+C to exit.");
    loop {
        thread::park();
    }
}